#![forbid(unsafe_code)]
//! VRF request binding.
//!
//! On-chain verification lives in the verification program and calls
//! `solana-ecvrf` directly. This crate builds the alpha string both sides
//! must use. Host proving is behind the `prove` feature and is not compiled
//! into the programs.

#![no_std]

use nuvex_common::MAX_INPUT_LEN;

pub use nuvex_crypto::{CryptoAvailability, VRF};

/// RFC 9381 ECVRF-EDWARDS25519-SHA512-TAI sizes from `solana-ecvrf` 0.0.1.
pub const VRF_PUBLIC_KEY_LEN: usize = 32;
pub const VRF_PROOF_LEN: usize = 80;
pub const VRF_OUTPUT_LEN: usize = 64;

/// Domain that keeps this alpha distinct from a raw user input.
pub const VRF_ALPHA_DOMAIN: &[u8] = b"nuvex-vrf-v1";

pub const MAX_VRF_ALPHA_LEN: usize = VRF_ALPHA_DOMAIN.len() + 32 + 1 + 2 + MAX_INPUT_LEN;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlphaError {
    BufferTooSmall,
    InputTooLarge,
}

/// `domain || request pubkey || job_type || input_len_le_u16 || input`.
///
/// The request pubkey is the account address, not the client-supplied id.
/// Instruction data must not be able to substitute a different alpha.
pub fn write_vrf_alpha(
    dst: &mut [u8],
    request: &[u8; 32],
    job_type: u8,
    input: &[u8],
) -> Result<usize, AlphaError> {
    if input.len() > MAX_INPUT_LEN {
        return Err(AlphaError::InputTooLarge);
    }
    let len = VRF_ALPHA_DOMAIN.len() + request.len() + 1 + 2 + input.len();
    if dst.len() < len {
        return Err(AlphaError::BufferTooSmall);
    }
    let mut cursor = 0;
    dst[cursor..cursor + VRF_ALPHA_DOMAIN.len()].copy_from_slice(VRF_ALPHA_DOMAIN);
    cursor += VRF_ALPHA_DOMAIN.len();
    dst[cursor..cursor + request.len()].copy_from_slice(request);
    cursor += request.len();
    dst[cursor] = job_type;
    cursor += 1;
    let input_len = u16::try_from(input.len()).map_err(|_| AlphaError::InputTooLarge)?;
    dst[cursor..cursor + 2].copy_from_slice(&input_len.to_le_bytes());
    cursor += 2;
    dst[cursor..cursor + input.len()].copy_from_slice(input);
    Ok(len)
}

#[cfg(feature = "prove")]
mod proving {
    use super::{VRF_OUTPUT_LEN, VRF_PROOF_LEN};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum ProveError {
        /// The library produced a proof that its own verifier rejected.
        Library,
    }

    pub fn public_key(secret: &[u8; 32]) -> [u8; 32] {
        solana_ecvrf::SecretKey(*secret).public_key().0
    }

    pub fn prove(
        secret: &[u8; 32],
        alpha: &[u8],
    ) -> Result<([u8; VRF_OUTPUT_LEN], [u8; VRF_PROOF_LEN]), ProveError> {
        let secret = solana_ecvrf::SecretKey(*secret);
        let proof = secret.prove(alpha);
        let output = proof
            .verify(&secret.public_key(), alpha)
            .map_err(|_| ProveError::Library)?;
        Ok((output, proof.0))
    }

    pub fn verify(
        public_key: &[u8; 32],
        alpha: &[u8],
        proof: &[u8; VRF_PROOF_LEN],
    ) -> Result<[u8; VRF_OUTPUT_LEN], ProveError> {
        solana_ecvrf::Proof(*proof)
            .verify(&solana_ecvrf::PublicKey(*public_key), alpha)
            .map_err(|_| ProveError::Library)
    }
}

#[cfg(feature = "prove")]
pub use proving::{prove, public_key, verify, ProveError};

#[cfg(test)]
mod tests {
    use super::{write_vrf_alpha, AlphaError, MAX_VRF_ALPHA_LEN, VRF_ALPHA_DOMAIN};
    use nuvex_common::MAX_INPUT_LEN;

    #[test]
    fn alpha_binds_the_request_and_rejects_an_oversized_input() {
        let request = [9u8; 32];
        let mut buf = [0u8; MAX_VRF_ALPHA_LEN];
        let len = write_vrf_alpha(&mut buf, &request, 1, b"dice").expect("alpha");
        assert!(buf[..len].starts_with(VRF_ALPHA_DOMAIN));
        assert_eq!(
            &buf[VRF_ALPHA_DOMAIN.len()..VRF_ALPHA_DOMAIN.len() + 32],
            &request
        );
        assert_eq!(buf[VRF_ALPHA_DOMAIN.len() + 32], 1);
        assert_eq!(
            &buf[VRF_ALPHA_DOMAIN.len() + 33..VRF_ALPHA_DOMAIN.len() + 35],
            &4u16.to_le_bytes()
        );
        assert!(write_vrf_alpha(&mut buf, &request, 1, &[0u8; MAX_INPUT_LEN + 1]).is_err());
        assert_eq!(
            write_vrf_alpha(&mut [0u8; 4], &request, 1, b"dice"),
            Err(AlphaError::BufferTooSmall)
        );
    }
}
