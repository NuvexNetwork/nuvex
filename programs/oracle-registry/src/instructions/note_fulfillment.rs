use anchor_lang::prelude::*;

use crate::constants::NODE_STATUS_ACTIVE;
use crate::errors::OracleRegistryError;
use crate::events::ReputationNoted;

pub fn handler(ctx: Context<crate::NoteFulfillment>) -> Result<()> {
    let node = &mut ctx.accounts.node;
    if node.status != NODE_STATUS_ACTIVE {
        return err!(OracleRegistryError::NodeNotActive);
    }
    node.reputation = node
        .reputation
        .checked_add(1)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;
    emit!(ReputationNoted {
        node: node.key(),
        reputation: node.reputation,
    });
    Ok(())
}
