use anchor_lang::prelude::*;

#[event]
pub struct VrfVerified {
    pub request: Pubkey,
    pub node: Pubkey,
    pub output: [u8; 64],
}
