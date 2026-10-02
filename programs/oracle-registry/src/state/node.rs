use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct NodeAccount {
    pub node_authority: Pubkey,
    pub operator: Pubkey,
    /// Ed25519 VRF key. All zeros cannot fulfill. A non-zero key is checked at
    /// registration and `update_node` cannot replace it.
    pub vrf_pubkey: [u8; nuvex_vrf::VRF_PUBLIC_KEY_LEN],
    /// Lamports locked in this account above rent.
    pub stake: u64,
    pub supported_job_mask: u32,
    pub status: u8,
    /// Count of successful fulfillments. Not a selection weight. Slash sets it to zero.
    pub reputation: u64,
    pub created_slot: u64,
    pub last_heartbeat: u64,
    pub bump: u8,
    /// Slot when a pending unstake may be withdrawn. Zero unless status is unstaking.
    pub cooldown_end_slot: u64,
}
