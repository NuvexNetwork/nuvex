use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::{AccountsExit, InstructionData, ToAccountMetas};
use nuvex_common::{write_callback_instruction, CALLBACK_OUTPUT_LEN};
use nuvex_job_types::JobType;
use nuvex_oracle_registry::constants::NODE_STATUS_ACTIVE;
use nuvex_protocol_types::RequestStatus;
use nuvex_verification::state::VrfResult;
use nuvex_vrf::{write_vrf_alpha, MAX_VRF_ALPHA_LEN, VRF_PROOF_LEN};

use crate::constants::{MAX_CALLBACK_DATA_LEN, PROTOCOL_SEED};
use crate::errors::OracleCoreError;
use crate::events::RequestFulfilled;
use crate::utils::{load_status, require_transition};

pub fn handler(ctx: Context<crate::Fulfill>, proof: [u8; VRF_PROOF_LEN]) -> Result<()> {
    let clock = Clock::get()?;
    let request_key = ctx.accounts.request.key();
    let node_key = ctx.accounts.node.key();
    let vrf_pubkey = ctx.accounts.node.vrf_pubkey;
    let node_created = ctx.accounts.node.created_slot;
    let mask = ctx.accounts.node.supported_job_mask;
    let node_status = ctx.accounts.node.status;
    let node_stake = ctx.accounts.node.stake;
    let last_heartbeat = ctx.accounts.node.last_heartbeat;
    let min_stake = ctx.accounts.registry.min_stake;
    let heartbeat_timeout = ctx.accounts.registry.heartbeat_timeout_slots;
    let cooldown_configured = ctx.accounts.registry.unstake_cooldown_slots;
    let protocol_bump = ctx.accounts.protocol.bump;

    let status = load_status(ctx.accounts.request.status)?;
    require_transition(status, RequestStatus::CallbackExecuted)?;
    if ctx.accounts.request.job_type != JobType::Vrf.as_u8() {
        return err!(OracleCoreError::RequestNotVrf);
    }
    if clock.slot >= ctx.accounts.request.expires_slot {
        return err!(OracleCoreError::RequestExpired);
    }
    if vrf_pubkey == [0u8; 32] || mask & JobType::Vrf.capability_bit() == 0 {
        return err!(OracleCoreError::NodeCannotFulfill);
    }
    if node_created >= ctx.accounts.request.created_slot {
        return err!(OracleCoreError::NodeNotEligible);
    }
    if cooldown_configured == 0 || heartbeat_timeout == 0 {
        return err!(OracleCoreError::StakeNotConfigured);
    }
    if node_status != NODE_STATUS_ACTIVE {
        return err!(OracleCoreError::NodeNotActive);
    }
    if node_stake < min_stake {
        return err!(OracleCoreError::StakeBelowMinimum);
    }
    if last_heartbeat == 0 {
        return err!(OracleCoreError::HeartbeatMissing);
    }
    let fresh_until = last_heartbeat
        .checked_add(heartbeat_timeout)
        .ok_or_else(|| error!(OracleCoreError::Overflow))?;
    if clock.slot >= fresh_until {
        return err!(OracleCoreError::HeartbeatStale);
    }

    let input_len = usize::from(ctx.accounts.request.input_len);
    if input_len > ctx.accounts.request.input.len() {
        return err!(OracleCoreError::InputTooLarge);
    }
    let mut alpha = [0u8; MAX_VRF_ALPHA_LEN];
    let alpha_len = write_vrf_alpha(
        &mut alpha,
        &request_key.to_bytes(),
        ctx.accounts.request.job_type,
        &ctx.accounts.request.input[..input_len],
    )
    .map_err(|_| error!(OracleCoreError::AlphaInvalid))?;

    let bump_seed = [protocol_bump];
    let signer_seeds: &[&[u8]] = &[PROTOCOL_SEED, &bump_seed];
    let ix_data = nuvex_verification::instruction::VerifyVrf {
        request: request_key,
        node: node_key,
        vrf_pubkey,
        alpha: alpha[..alpha_len].to_vec(),
        proof,
    }
    .data();
    let ix_accounts = nuvex_verification::accounts::VerifyVrf {
        protocol: ctx.accounts.protocol.key(),
        payer: ctx.accounts.node_authority.key(),
        result: ctx.accounts.vrf_result.key(),
        system_program: ctx.accounts.system_program.key(),
    }
    .to_account_metas(None);
    invoke_signed(
        &Instruction {
            program_id: nuvex_verification::ID,
            accounts: ix_accounts,
            data: ix_data,
        },
        &[
            ctx.accounts.protocol.to_account_info(),
            ctx.accounts.node_authority.to_account_info(),
            ctx.accounts.vrf_result.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.verification_program.to_account_info(),
        ],
        &[signer_seeds],
    )?;

    let note = nuvex_oracle_registry::instruction::NoteFulfillment {}.data();
    let note_accounts = nuvex_oracle_registry::accounts::NoteFulfillment {
        protocol: ctx.accounts.protocol.key(),
        node: node_key,
    }
    .to_account_metas(None);
    invoke_signed(
        &Instruction {
            program_id: nuvex_oracle_registry::ID,
            accounts: note_accounts,
            data: note,
        },
        &[
            ctx.accounts.protocol.to_account_info(),
            ctx.accounts.node.to_account_info(),
            ctx.accounts.registry_program.to_account_info(),
        ],
        &[signer_seeds],
    )?;
    ctx.accounts.node.reload()?;

    let output = {
        require_keys_eq!(
            *ctx.accounts.vrf_result.owner,
            nuvex_verification::ID,
            OracleCoreError::VerificationMismatch
        );
        let data = ctx.accounts.vrf_result.try_borrow_data()?;
        let stored = VrfResult::try_deserialize(&mut &data[..])
            .map_err(|_| error!(OracleCoreError::VerificationMismatch))?;
        if stored.request != request_key
            || stored.node != node_key
            || stored.vrf_pubkey != vrf_pubkey
            || stored.proof != proof
        {
            return err!(OracleCoreError::VerificationMismatch);
        }
        stored.output
    };

    let callback_program = ctx.accounts.request.callback_program;
    let callback_len = usize::from(ctx.accounts.request.callback_data_len);
    if callback_len > ctx.accounts.request.callback_data.len() {
        return err!(OracleCoreError::CallbackDataTooLarge);
    }
    let mut callback_data = [0u8; MAX_CALLBACK_DATA_LEN];
    callback_data[..callback_len]
        .copy_from_slice(&ctx.accounts.request.callback_data[..callback_len]);

    ctx.accounts.request.status = RequestStatus::CallbackExecuted.as_u8();
    ctx.accounts.request.assigned_node = node_key;
    ctx.accounts.request.assigned_stake = node_stake;
    ctx.accounts.request.assigned_heartbeat = last_heartbeat;
    ctx.accounts.request.exit(&crate::ID)?;

    if callback_program == Pubkey::default() {
        if ctx.accounts.callback_program.is_some() {
            return err!(OracleCoreError::CallbackProgramMismatch);
        }
    } else {
        let program = ctx
            .accounts
            .callback_program
            .as_ref()
            .ok_or_else(|| error!(OracleCoreError::CallbackProgramMismatch))?;
        if program.key() != callback_program || !program.executable {
            return err!(OracleCoreError::CallbackProgramMismatch);
        }
        let mut ix_data = [0u8; 8 + CALLBACK_OUTPUT_LEN + 4 + MAX_CALLBACK_DATA_LEN];
        let written =
            write_callback_instruction(&mut ix_data, &output, &callback_data[..callback_len])
                .map_err(|_| error!(OracleCoreError::CallbackDataTooLarge))?;
        anchor_lang::solana_program::program::invoke(
            &Instruction {
                program_id: callback_program,
                accounts: Vec::new(),
                data: ix_data[..written].to_vec(),
            },
            &[program.to_account_info()],
        )?;
    }

    emit!(RequestFulfilled {
        request: request_key,
        node: node_key,
        output,
    });
    Ok(())
}
