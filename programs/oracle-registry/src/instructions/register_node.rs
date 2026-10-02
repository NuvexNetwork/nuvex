use anchor_lang::prelude::*;
use nuvex_job_types::mask_is_known;

use crate::constants::NODE_STATUS_REGISTERED;
use crate::errors::OracleRegistryError;
use crate::events::NodeRegistered;

const _: () = assert!(solana_ecvrf::PUBLIC_KEY_LENGTH == nuvex_vrf::VRF_PUBLIC_KEY_LEN);

pub fn handler(
    ctx: Context<crate::RegisterNode>,
    operator: Pubkey,
    vrf_pubkey: [u8; 32],
    supported_job_mask: u32,
) -> Result<()> {
    if operator == Pubkey::default() {
        return err!(OracleRegistryError::InvalidOperator);
    }
    if !mask_is_known(supported_job_mask) {
        return err!(OracleRegistryError::InvalidJobMask);
    }
    if vrf_pubkey != [0u8; 32] {
        solana_ecvrf::PublicKey(vrf_pubkey)
            .validate()
            .map_err(|_| error!(OracleRegistryError::InvalidVrfKey))?;
    }

    let clock = Clock::get()?;
    let node = &mut ctx.accounts.node;
    node.node_authority = ctx.accounts.node_authority.key();
    node.operator = operator;
    node.vrf_pubkey = vrf_pubkey;
    node.stake = 0;
    node.supported_job_mask = supported_job_mask;
    node.status = NODE_STATUS_REGISTERED;
    node.reputation = 0;
    node.created_slot = clock.slot;
    node.last_heartbeat = 0;
    node.bump = ctx.bumps.node;
    node.cooldown_end_slot = 0;

    let registry = &mut ctx.accounts.registry;
    registry.node_count = registry
        .node_count
        .checked_add(1)
        .ok_or_else(|| error!(OracleRegistryError::Overflow))?;

    emit!(NodeRegistered {
        node: node.key(),
        node_authority: node.node_authority,
        operator,
        vrf_pubkey,
        supported_job_mask,
    });
    Ok(())
}
