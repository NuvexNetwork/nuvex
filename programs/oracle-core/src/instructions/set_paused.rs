use anchor_lang::prelude::*;

use crate::events::ProtocolPauseSet;

pub fn handler(ctx: Context<crate::SetPaused>, paused: bool) -> Result<()> {
    ctx.accounts.protocol.paused = paused;
    emit!(ProtocolPauseSet { paused });
    Ok(())
}
