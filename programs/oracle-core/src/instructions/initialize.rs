use anchor_lang::prelude::*;

use crate::events::ProtocolInitialized;

pub fn handler(ctx: Context<crate::Initialize>) -> Result<()> {
    let protocol = &mut ctx.accounts.protocol;
    protocol.authority = ctx.accounts.authority.key();
    protocol.paused = false;
    protocol.bump = ctx.bumps.protocol;
    emit!(ProtocolInitialized {
        authority: protocol.authority,
    });
    Ok(())
}
