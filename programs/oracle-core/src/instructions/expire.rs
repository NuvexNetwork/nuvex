use anchor_lang::prelude::*;
use nuvex_protocol_types::RequestStatus;

use crate::errors::OracleCoreError;
use crate::events::RequestExpired;
use crate::utils::{load_status, require_transition};

pub fn handler(ctx: Context<crate::Expire>) -> Result<()> {
    let status = load_status(ctx.accounts.request.status)?;
    require_transition(status, RequestStatus::Expired)?;
    let clock = Clock::get()?;
    if clock.slot < ctx.accounts.request.expires_slot {
        return err!(OracleCoreError::NotExpired);
    }
    let slot = clock.slot;
    let request = &mut ctx.accounts.request;
    request.status = RequestStatus::Expired.as_u8();
    emit!(RequestExpired {
        request: request.key(),
        slot,
    });
    Ok(())
}
