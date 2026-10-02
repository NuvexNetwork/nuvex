use anchor_lang::prelude::*;

use crate::constants::{NODE_STATUS_ACTIVE, NODE_STATUS_REGISTERED, NODE_STATUS_UNSTAKING};
use crate::errors::OracleRegistryError;
use crate::events::NodeSlashed;
use crate::utils::move_lamports;

pub fn handler(ctx: Context<crate::Slash>, amount: u64) -> Result<()> {
    if ctx.accounts.registry.slash_authority == Pubkey::default() {
        return err!(OracleRegistryError::SlashDisabled);
    }
    if ctx.accounts.slash_authority.key() != ctx.accounts.registry.slash_authority {
        return err!(OracleRegistryError::InvalidSlashAuthority);
    }
    if ctx.accounts.destination.key() != ctx.accounts.registry.slash_destination {
        return err!(OracleRegistryError::InvalidSlashDestination);
    }
    if amount == 0 {
        return err!(OracleRegistryError::StakeAmountZero);
    }
    let status = ctx.accounts.node.status;
    if status != NODE_STATUS_ACTIVE && status != NODE_STATUS_UNSTAKING {
        return err!(OracleRegistryError::InvalidStakeStatus);
    }
    if amount > ctx.accounts.node.stake {
        return err!(OracleRegistryError::SlashExceedsStake);
    }

    move_lamports(
        &ctx.accounts.node.to_account_info(),
        &ctx.accounts.destination.to_account_info(),
        amount,
    )?;

    let node = &mut ctx.accounts.node;
    node.stake = node
        .stake
        .checked_sub(amount)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;
    node.reputation = 0;
    if node.stake == 0 {
        node.status = NODE_STATUS_REGISTERED;
        node.cooldown_end_slot = 0;
    }

    emit!(NodeSlashed {
        node: node.key(),
        amount,
        stake: node.stake,
    });
    Ok(())
}
