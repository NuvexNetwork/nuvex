use anchor_lang::prelude::*;

use crate::constants::{NODE_STATUS_REGISTERED, NODE_STATUS_UNSTAKING};
use crate::errors::OracleRegistryError;
use crate::events::StakeWithdrawn;
use crate::utils::move_lamports;

pub fn handler(ctx: Context<crate::Withdraw>) -> Result<()> {
    if ctx.accounts.node.status != NODE_STATUS_UNSTAKING {
        return err!(OracleRegistryError::NotUnstaking);
    }
    let clock = Clock::get()?;
    if clock.slot < ctx.accounts.node.cooldown_end_slot {
        return err!(OracleRegistryError::CooldownNotElapsed);
    }

    let amount = ctx.accounts.node.stake;
    move_lamports(
        &ctx.accounts.node.to_account_info(),
        &ctx.accounts.node_authority.to_account_info(),
        amount,
    )?;

    let node = &mut ctx.accounts.node;
    node.stake = 0;
    node.status = NODE_STATUS_REGISTERED;
    node.cooldown_end_slot = 0;

    emit!(StakeWithdrawn {
        node: node.key(),
        amount,
    });
    Ok(())
}
