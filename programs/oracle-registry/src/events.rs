use anchor_lang::prelude::*;

#[event]
pub struct RegistryInitialized {
    pub authority: Pubkey,
}

#[event]
pub struct NodeRegistered {
    pub node: Pubkey,
    pub node_authority: Pubkey,
    pub operator: Pubkey,
    pub vrf_pubkey: [u8; 32],
    pub supported_job_mask: u32,
}

#[event]
pub struct NodeUpdated {
    pub node: Pubkey,
    pub operator: Pubkey,
    pub supported_job_mask: u32,
}

#[event]
pub struct StakeConfigured {
    pub min_stake: u64,
    pub unstake_cooldown_slots: u64,
    pub heartbeat_timeout_slots: u64,
    pub slash_authority: Pubkey,
    pub slash_destination: Pubkey,
}

#[event]
pub struct StakeDeposited {
    pub node: Pubkey,
    pub amount: u64,
    pub stake: u64,
}

#[event]
pub struct UnstakeRequested {
    pub node: Pubkey,
    pub cooldown_end_slot: u64,
}

#[event]
pub struct StakeWithdrawn {
    pub node: Pubkey,
    pub amount: u64,
}

#[event]
pub struct NodeHeartbeat {
    pub node: Pubkey,
    pub slot: u64,
}

#[event]
pub struct NodeSlashed {
    pub node: Pubkey,
    pub amount: u64,
    pub stake: u64,
}

#[event]
pub struct ReputationNoted {
    pub node: Pubkey,
    pub reputation: u64,
}
