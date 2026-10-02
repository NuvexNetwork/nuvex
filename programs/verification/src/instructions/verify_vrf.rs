use anchor_lang::prelude::*;
use nuvex_vrf::{MAX_VRF_ALPHA_LEN, VRF_ALPHA_DOMAIN};

use crate::errors::VerificationError;
use crate::events::VrfVerified;

const _: () = assert!(solana_ecvrf::PROOF_LENGTH == nuvex_vrf::VRF_PROOF_LEN);
const _: () = assert!(solana_ecvrf::OUTPUT_LENGTH == nuvex_vrf::VRF_OUTPUT_LEN);
const _: () = assert!(solana_ecvrf::PUBLIC_KEY_LENGTH == nuvex_vrf::VRF_PUBLIC_KEY_LEN);

pub fn handler(
    ctx: Context<crate::VerifyVrf>,
    request: Pubkey,
    node: Pubkey,
    vrf_pubkey: [u8; 32],
    alpha: Vec<u8>,
    proof: [u8; 80],
) -> Result<()> {
    if alpha.len() > MAX_VRF_ALPHA_LEN || !alpha.starts_with(VRF_ALPHA_DOMAIN) {
        return err!(VerificationError::AlphaMalformed);
    }

    let output = solana_ecvrf::Proof(proof)
        .verify(&solana_ecvrf::PublicKey(vrf_pubkey), &alpha)
        .map_err(|error| match error {
            solana_ecvrf::Error::InvalidPublicKey => error!(VerificationError::InvalidPublicKey),
            solana_ecvrf::Error::InvalidProof => error!(VerificationError::InvalidProof),
        })?;

    let result = &mut ctx.accounts.result;
    result.request = request;
    result.node = node;
    result.vrf_pubkey = vrf_pubkey;
    result.output = output;
    result.proof = proof;
    result.bump = ctx.bumps.result;

    emit!(VrfVerified {
        request,
        node,
        output,
    });
    Ok(())
}
