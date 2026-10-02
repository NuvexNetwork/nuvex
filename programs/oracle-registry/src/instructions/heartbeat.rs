use anchor_lang::prelude::*;

use crate::constants::NODE_STATUS_ACTIVE;
use crate::errors::OracleRegistryError;
use crate::events::NodeHeartbeat;

pub fn handler(ctx: Context<crate::Heartbeat>) -> Result<()> {
    if ctx.accounts.node.status != NODE_STATUS_ACTIVE {
        return err!(OracleRegistryError::HeartbeatNotActive);
    }
    let slot = Clock::get()?.slot;
    let node = &mut ctx.accounts.node;
    node.last_heartbeat = slot;
    emit!(NodeHeartbeat {
        node: node.key(),
        slot,
    });
    Ok(())
}
