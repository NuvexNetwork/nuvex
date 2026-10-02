use anchor_lang::prelude::*;

use crate::events::RegistryInitialized;

pub fn handler(ctx: Context<crate::Initialize>) -> Result<()> {
    let registry = &mut ctx.accounts.registry;
    registry.authority = ctx.accounts.authority.key();
    registry.node_count = 0;
    registry.bump = ctx.bumps.registry;
    registry.min_stake = 0;
    registry.unstake_cooldown_slots = 0;
    registry.heartbeat_timeout_slots = 0;
    registry.slash_authority = Pubkey::default();
    registry.slash_destination = Pubkey::default();
    emit!(RegistryInitialized {
        authority: registry.authority,
    });
    Ok(())
}
