use anchor_lang::prelude::*;

use crate::errors::OracleRegistryError;
use crate::events::StakeConfigured;
use crate::utils::slot_window;

pub fn handler(
    ctx: Context<crate::ConfigureStake>,
    min_stake: u64,
    unstake_cooldown_slots: u64,
    heartbeat_timeout_slots: u64,
    slash_authority: Pubkey,
    slash_destination: Pubkey,
) -> Result<()> {
    slot_window(unstake_cooldown_slots)?;
    slot_window(heartbeat_timeout_slots)?;
    let slash_set = slash_authority != Pubkey::default();
    if slash_set != (slash_destination != Pubkey::default()) {
        return err!(OracleRegistryError::InvalidSlashConfig);
    }

    let registry = &mut ctx.accounts.registry;
    registry.min_stake = min_stake;
    registry.unstake_cooldown_slots = unstake_cooldown_slots;
    registry.heartbeat_timeout_slots = heartbeat_timeout_slots;
    registry.slash_authority = slash_authority;
    registry.slash_destination = slash_destination;

    emit!(StakeConfigured {
        min_stake,
        unstake_cooldown_slots,
        heartbeat_timeout_slots,
        slash_authority,
        slash_destination,
    });
    Ok(())
}
