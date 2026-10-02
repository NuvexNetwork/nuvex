//! Shared checks for stake instructions.

use anchor_lang::prelude::*;
use nuvex_common::MAX_TIMEOUT_SLOTS;

use crate::errors::OracleRegistryError;
use crate::state::NodeRegistry;

pub fn require_configured(registry: &NodeRegistry) -> Result<()> {
    if registry.unstake_cooldown_slots == 0 || registry.heartbeat_timeout_slots == 0 {
        return err!(OracleRegistryError::StakeNotConfigured);
    }
    Ok(())
}

pub fn slot_window(slots: u64) -> Result<()> {
    if slots == 0 || slots > MAX_TIMEOUT_SLOTS {
        return err!(OracleRegistryError::InvalidStakeParameter);
    }
    Ok(())
}

pub fn move_lamports<'info>(
    from: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let rent = Rent::get()?;
    let minimum = rent.minimum_balance(from.data_len());
    let next = from
        .lamports()
        .checked_sub(amount)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;
    if next < minimum {
        return err!(OracleRegistryError::InsufficientStake);
    }
    **from.try_borrow_mut_lamports()? = next;
    let credited = to
        .lamports()
        .checked_add(amount)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;
    **to.try_borrow_mut_lamports()? = credited;
    Ok(())
}
