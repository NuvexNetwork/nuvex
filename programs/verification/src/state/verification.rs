use anchor_lang::prelude::*;

/// Proof material for one request. Oracle core is the only creator, because
/// the protocol PDA must sign the instruction that initializes this account.
#[account]
#[derive(InitSpace)]
pub struct VrfResult {
    pub request: Pubkey,
    pub node: Pubkey,
    pub vrf_pubkey: [u8; nuvex_vrf::VRF_PUBLIC_KEY_LEN],
    pub output: [u8; nuvex_vrf::VRF_OUTPUT_LEN],
    pub proof: [u8; nuvex_vrf::VRF_PROOF_LEN],
    pub bump: u8,
}
