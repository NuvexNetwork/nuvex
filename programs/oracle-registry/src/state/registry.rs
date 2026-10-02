use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct NodeRegistry {
    pub authority: Pubkey,
    /// Incremented on each registration. This account is a write hotspot.
    pub node_count: u64,
    pub bump: u8,
    /// Lamports required before a node becomes active. Zero is a valid config
    /// and does not resist Sybil.
    pub min_stake: u64,
    /// Slots added to the current slot when unstake starts. Zero means unset.
    pub unstake_cooldown_slots: u64,
    /// A heartbeat is fresh while `slot < last_heartbeat + this`. Zero means unset.
    pub heartbeat_timeout_slots: u64,
    /// Default pubkey disables slash.
    pub slash_authority: Pubkey,
    /// Receives slashed lamports. Default pubkey disables slash.
    pub slash_destination: Pubkey,
}
