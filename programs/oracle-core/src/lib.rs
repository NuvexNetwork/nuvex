//! Oracle core program.
//!
//! Milestone 3 fulfills a pending VRF request only for an active node whose
//! stake and heartbeat meet the registry configuration. Fees are stored and
//! never transferred.

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

use constants::{JOB_CONFIG_SEED, NODE_SEED, PROTOCOL_SEED, REGISTRY_SEED, REQUEST_SEED};
use errors::OracleCoreError;
use nuvex_oracle_registry::state::{NodeAccount, NodeRegistry};
use state::{JobConfig, OracleRequest, ProtocolConfig};

declare_id!("9CHifhFPRK8nJJpL2nTY17UfYC75aReR4qojVqNaLnoe");

#[program]
pub mod oracle_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn set_paused(ctx: Context<SetPaused>, paused: bool) -> Result<()> {
        instructions::set_paused::handler(ctx, paused)
    }

    pub fn configure_job(
        ctx: Context<ConfigureJob>,
        job_type: u8,
        timeout_slots: u64,
        requests_enabled: bool,
    ) -> Result<()> {
        instructions::configure_job::handler(ctx, job_type, timeout_slots, requests_enabled)
    }

    pub fn update_job(
        ctx: Context<UpdateJob>,
        job_type: u8,
        timeout_slots: u64,
        requests_enabled: bool,
    ) -> Result<()> {
        instructions::update_job::handler(ctx, job_type, timeout_slots, requests_enabled)
    }

    /// Opens a request. The account type is `CreateRequest` so it does not
    /// share a name with the request account module.
    pub fn request(
        ctx: Context<CreateRequest>,
        request_id: [u8; 32],
        job_type: u8,
        input: Vec<u8>,
        max_fee: u64,
        callback_program: Pubkey,
        callback_data: Vec<u8>,
    ) -> Result<()> {
        instructions::request::handler(
            ctx,
            request_id,
            job_type,
            input,
            max_fee,
            callback_program,
            callback_data,
        )
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        instructions::cancel::handler(ctx)
    }

    pub fn expire(ctx: Context<Expire>) -> Result<()> {
        instructions::expire::handler(ctx)
    }

    pub fn close_request(ctx: Context<CloseRequest>) -> Result<()> {
        instructions::close_request::handler(ctx)
    }

    pub fn fulfill(ctx: Context<Fulfill>, proof: [u8; nuvex_vrf::VRF_PROOF_LEN]) -> Result<()> {
        instructions::fulfill::handler(ctx, proof)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + ProtocolConfig::INIT_SPACE,
        seeds = [PROTOCOL_SEED],
        bump
    )]
    pub protocol: Account<'info, ProtocolConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPaused<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [PROTOCOL_SEED],
        bump = protocol.bump,
        has_one = authority
    )]
    pub protocol: Account<'info, ProtocolConfig>,
}

#[derive(Accounts)]
#[instruction(job_type: u8)]
pub struct ConfigureJob<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol.bump,
        has_one = authority
    )]
    pub protocol: Account<'info, ProtocolConfig>,
    #[account(
        init,
        payer = authority,
        space = 8 + JobConfig::INIT_SPACE,
        seeds = [JOB_CONFIG_SEED, &[job_type]],
        bump
    )]
    pub job_config: Account<'info, JobConfig>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(job_type: u8)]
pub struct UpdateJob<'info> {
    pub authority: Signer<'info>,
    #[account(
        seeds = [PROTOCOL_SEED],
        bump = protocol.bump,
        has_one = authority
    )]
    pub protocol: Account<'info, ProtocolConfig>,
    #[account(
        mut,
        seeds = [JOB_CONFIG_SEED, &[job_type]],
        bump = job_config.bump
    )]
    pub job_config: Account<'info, JobConfig>,
}

#[derive(Accounts)]
#[instruction(request_id: [u8; 32], job_type: u8)]
pub struct CreateRequest<'info> {
    #[account(mut)]
    pub requester: Signer<'info>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol.bump)]
    pub protocol: Account<'info, ProtocolConfig>,
    #[account(seeds = [JOB_CONFIG_SEED, &[job_type]], bump = job_config.bump)]
    pub job_config: Account<'info, JobConfig>,
    #[account(
        init,
        payer = requester,
        space = 8 + OracleRequest::INIT_SPACE,
        seeds = [REQUEST_SEED, requester.key().as_ref(), request_id.as_ref()],
        bump
    )]
    pub request: Account<'info, OracleRequest>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    pub requester: Signer<'info>,
    #[account(
        mut,
        seeds = [REQUEST_SEED, request.requester.as_ref(), request.request_id.as_ref()],
        bump = request.bump,
        has_one = requester
    )]
    pub request: Account<'info, OracleRequest>,
}

#[derive(Accounts)]
pub struct Expire<'info> {
    pub crank: Signer<'info>,
    #[account(
        mut,
        seeds = [REQUEST_SEED, request.requester.as_ref(), request.request_id.as_ref()],
        bump = request.bump
    )]
    pub request: Account<'info, OracleRequest>,
}

#[derive(Accounts)]
pub struct CloseRequest<'info> {
    pub crank: Signer<'info>,
    #[account(
        mut,
        seeds = [REQUEST_SEED, request.requester.as_ref(), request.request_id.as_ref()],
        bump = request.bump,
        close = requester
    )]
    pub request: Account<'info, OracleRequest>,
    #[account(mut, address = request.requester)]
    pub requester: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct Fulfill<'info> {
    #[account(mut)]
    pub node_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump = node.bump,
        seeds::program = nuvex_oracle_registry::ID,
        constraint = node.node_authority == node_authority.key() @ OracleCoreError::NodeCannotFulfill
    )]
    pub node: Account<'info, NodeAccount>,
    #[account(
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        seeds::program = nuvex_oracle_registry::ID
    )]
    pub registry: Account<'info, NodeRegistry>,
    #[account(
        mut,
        seeds = [REQUEST_SEED, request.requester.as_ref(), request.request_id.as_ref()],
        bump = request.bump
    )]
    pub request: Account<'info, OracleRequest>,
    #[account(seeds = [PROTOCOL_SEED], bump = protocol.bump)]
    pub protocol: Account<'info, ProtocolConfig>,
    pub verification_program: Program<'info, nuvex_verification::program::Verification>,
    /// CHECK: initialized by the verification program during this instruction.
    #[account(mut)]
    pub vrf_result: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub registry_program: Program<'info, nuvex_oracle_registry::program::OracleRegistry>,
    /// CHECK: present only when the request stored a callback program.
    pub callback_program: Option<UncheckedAccount<'info>>,
}

#[cfg(test)]
mod space_tests {
    use anchor_lang::Space;

    use super::state::{JobConfig, OracleRequest, ProtocolConfig};

    #[test]
    fn account_spaces_match_the_field_layout() {
        assert_eq!(ProtocolConfig::INIT_SPACE, 34);
        assert_eq!(JobConfig::INIT_SPACE, 11);
        assert_eq!(OracleRequest::INIT_SPACE, 559);
    }

    #[test]
    fn verification_names_this_program() {
        assert_eq!(nuvex_verification::ORACLE_CORE_ID, crate::ID);
        assert_eq!(nuvex_oracle_registry::ORACLE_CORE_ID, crate::ID);
    }
}
