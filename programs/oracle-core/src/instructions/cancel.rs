use anchor_lang::prelude::*;
use nuvex_protocol_types::RequestStatus;

use crate::errors::OracleCoreError;
use crate::events::RequestCancelled;
use crate::utils::{load_status, require_transition};

pub fn handler(ctx: Context<crate::Cancel>) -> Result<()> {
    let status = load_status(ctx.accounts.request.status)?;
    require_transition(status, RequestStatus::Cancelled)?;
    let clock = Clock::get()?;
    if clock.slot >= ctx.accounts.request.expires_slot {
        return err!(OracleCoreError::RequestExpired);
    }
    let request = &mut ctx.accounts.request;
    request.status = RequestStatus::Cancelled.as_u8();
    emit!(RequestCancelled {
        request: request.key(),
        requester: request.requester,
    });
    Ok(())
}
