use anchor_lang::prelude::*;

#[error_code]
pub enum VerificationError {
    #[msg("The VRF alpha is not the Nuvex binding")]
    AlphaMalformed,
    #[msg("The VRF public key is not a valid Ed25519 point")]
    InvalidPublicKey,
    #[msg("The VRF proof does not verify")]
    InvalidProof,
}
