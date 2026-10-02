use anchor_lang::prelude::*;

use crate::errors::OracleCoreError;
use crate::events::RequestClosed;
use crate::utils::load_status;

pub fn handler(ctx: Context<crate::CloseRequest>) -> Result<()> {
    let status = load_status(ctx.accounts.request.status)?;
    if !status.is_terminal() {
        return err!(OracleCoreError::NotTerminal);
    }
    emit!(RequestClosed {
        request: ctx.accounts.request.key(),
        requester: ctx.accounts.request.requester,
    });
    Ok(())
}
