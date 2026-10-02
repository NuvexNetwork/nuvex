use anchor_lang::prelude::*;

use crate::constants::{NODE_STATUS_ACTIVE, NODE_STATUS_REGISTERED, NODE_STATUS_UNSTAKING};
use crate::errors::OracleRegistryError;
use crate::events::UnstakeRequested;
use crate::utils::require_configured;

pub fn handler(ctx: Context<crate::Unstake>) -> Result<()> {
    require_configured(&ctx.accounts.registry)?;
    let status = ctx.accounts.node.status;
    if status != NODE_STATUS_REGISTERED && status != NODE_STATUS_ACTIVE {
        return err!(OracleRegistryError::InvalidStakeStatus);
    }
    if ctx.accounts.node.stake == 0 {
        return err!(OracleRegistryError::StakeAmountZero);
    }

    let clock = Clock::get()?;
    let cooldown_end_slot = clock
        .slot
        .checked_add(ctx.accounts.registry.unstake_cooldown_slots)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;

    let node = &mut ctx.accounts.node;
    node.status = NODE_STATUS_UNSTAKING;
    node.cooldown_end_slot = cooldown_end_slot;

    emit!(UnstakeRequested {
        node: node.key(),
        cooldown_end_slot,
    });
    Ok(())
}
