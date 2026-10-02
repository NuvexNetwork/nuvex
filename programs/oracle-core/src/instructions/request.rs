use anchor_lang::prelude::*;
use nuvex_protocol_types::RequestStatus;

use crate::errors::OracleCoreError;
use crate::events::RequestCreated;
use crate::instructions::configure_job::require_requestable;

pub fn handler(
    ctx: Context<crate::CreateRequest>,
    request_id: [u8; 32],
    job_type: u8,
    input: Vec<u8>,
    max_fee: u64,
    callback_program: Pubkey,
    callback_data: Vec<u8>,
) -> Result<()> {
    if ctx.accounts.protocol.paused {
        return err!(OracleCoreError::ProtocolPaused);
    }
    require_requestable(job_type)?;
    if ctx.accounts.job_config.job_type != job_type {
        return err!(OracleCoreError::UnsupportedJob);
    }
    if !ctx.accounts.job_config.requests_enabled {
        return err!(OracleCoreError::JobRequestsDisabled);
    }
    if input.len() > ctx.accounts.request.input.len() {
        return err!(OracleCoreError::InputTooLarge);
    }
    if callback_data.len() > ctx.accounts.request.callback_data.len() {
        return err!(OracleCoreError::CallbackDataTooLarge);
    }

    let clock = Clock::get()?;
    let expires_slot = clock
        .slot
        .checked_add(ctx.accounts.job_config.timeout_slots)
        .ok_or_else(|| error!(OracleCoreError::Overflow))?;

    let request = &mut ctx.accounts.request;
    request.requester = ctx.accounts.requester.key();
    request.job_type = job_type;
    request.status = RequestStatus::Pending.as_u8();
    // Recorded only. No lamports move for this value.
    request.max_fee = max_fee;
    request.created_slot = clock.slot;
    request.expires_slot = expires_slot;
    request.request_id = request_id;
    request.input_len =
        u16::try_from(input.len()).map_err(|_| error!(OracleCoreError::Overflow))?;
    request.input[..input.len()].copy_from_slice(&input);
    request.callback_program = callback_program;
    request.callback_data_len =
        u16::try_from(callback_data.len()).map_err(|_| error!(OracleCoreError::Overflow))?;
    request.callback_data[..callback_data.len()].copy_from_slice(&callback_data);
    request.bump = ctx.bumps.request;
    request.assigned_node = Pubkey::default();
    request.assigned_stake = 0;
    request.assigned_heartbeat = 0;

    emit!(RequestCreated {
        request: request.key(),
        requester: request.requester,
        job_type,
        expires_slot,
        max_fee,
    });
    Ok(())
}
