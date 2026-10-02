use anchor_lang::prelude::*;

#[event]
pub struct ProtocolInitialized {
    pub authority: Pubkey,
}

#[event]
pub struct ProtocolPauseSet {
    pub paused: bool,
}

#[event]
pub struct JobConfigured {
    pub job_type: u8,
    pub timeout_slots: u64,
    pub requests_enabled: bool,
}

#[event]
pub struct RequestCreated {
    pub request: Pubkey,
    pub requester: Pubkey,
    pub job_type: u8,
    pub expires_slot: u64,
    pub max_fee: u64,
}

#[event]
pub struct RequestCancelled {
    pub request: Pubkey,
    pub requester: Pubkey,
}

#[event]
pub struct RequestExpired {
    pub request: Pubkey,
    pub slot: u64,
}

#[event]
pub struct RequestClosed {
    pub request: Pubkey,
    pub requester: Pubkey,
}

#[event]
pub struct RequestFulfilled {
    pub request: Pubkey,
    pub node: Pubkey,
    pub output: [u8; 64],
}
