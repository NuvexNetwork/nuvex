use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;

use crate::constants::{NODE_STATUS_ACTIVE, NODE_STATUS_REGISTERED};
use crate::errors::OracleRegistryError;
use crate::events::StakeDeposited;
use crate::utils::require_configured;

pub fn handler(ctx: Context<crate::Stake>, amount: u64) -> Result<()> {
    if amount == 0 {
        return err!(OracleRegistryError::StakeAmountZero);
    }
    require_configured(&ctx.accounts.registry)?;
    let status = ctx.accounts.node.status;
    if status != NODE_STATUS_REGISTERED && status != NODE_STATUS_ACTIVE {
        return err!(OracleRegistryError::InvalidStakeStatus);
    }

    let from = ctx.accounts.node_authority.key();
    invoke(
        &system_instruction::transfer(&from, &ctx.accounts.node.key(), amount),
        &[
            ctx.accounts.node_authority.to_account_info(),
            ctx.accounts.node.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    let min_stake = ctx.accounts.registry.min_stake;
    let node = &mut ctx.accounts.node;
    node.stake = node
        .stake
        .checked_add(amount)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;
    if node.status == NODE_STATUS_REGISTERED && node.stake >= min_stake {
        node.status = NODE_STATUS_ACTIVE;
    }

    emit!(StakeDeposited {
        node: node.key(),
        amount,
        stake: node.stake,
    });
    Ok(())
}
