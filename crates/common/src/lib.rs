#![forbid(unsafe_code)]
//! Shared constants for Nuvex.
//!
//! This crate is `no_std` so on-chain programs can depend on it. It contains
//! PDA seeds and bounds only. It does not create accounts or move value.

#![no_std]

/// Maximum Solana PDA seed length.
pub const MAX_SEED_LEN: usize = 32;

pub const PROTOCOL_SEED: &[u8] = b"protocol";
pub const REGISTRY_SEED: &[u8] = b"registry";
pub const NODE_SEED: &[u8] = b"node";
pub const REQUEST_SEED: &[u8] = b"request";
pub const VERIFICATION_SEED: &[u8] = b"verification";
pub const CHALLENGE_SEED: &[u8] = b"challenge";
pub const REWARD_VAULT_SEED: &[u8] = b"reward_vault";
pub const JOB_CONFIG_SEED: &[u8] = b"job_config";

/// Request ids are fixed-width so PDA seeds do not depend on a length prefix.
pub const REQUEST_ID_LEN: usize = 32;
pub const MAX_INPUT_LEN: usize = 256;
pub const MAX_CALLBACK_DATA_LEN: usize = 128;

/// Safety cap so a job cannot be configured to never expire.
/// This is not a fee and not a stake parameter.
pub const MIN_TIMEOUT_SLOTS: u64 = 1;
pub const MAX_TIMEOUT_SLOTS: u64 = 150_000;

/// Every published seed must fit in a single PDA seed.
pub const fn seed_is_valid(seed: &[u8]) -> bool {
    !seed.is_empty() && seed.len() <= MAX_SEED_LEN
}

/// First 8 bytes of SHA-256("global:callback").
///
/// This is the Anchor discriminator for an instruction named `callback`.
/// The payload after it is a 64-byte VRF output, a little-endian `u32` length,
/// and the stored callback data. The account list is empty.
pub const CALLBACK_IX_DISCRIMINATOR: [u8; 8] = [245, 250, 10, 62, 218, 252, 239, 91];

/// VRF output carried in a callback. Matches the proof format in `nuvex-vrf`.
pub const CALLBACK_OUTPUT_LEN: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallbackIxError;

pub fn write_callback_instruction(
    dst: &mut [u8],
    output: &[u8; CALLBACK_OUTPUT_LEN],
    callback_data: &[u8],
) -> Result<usize, CallbackIxError> {
    let len = CALLBACK_IX_DISCRIMINATOR
        .len()
        .saturating_add(CALLBACK_OUTPUT_LEN)
        .saturating_add(4)
        .saturating_add(callback_data.len());
    if callback_data.len() > MAX_CALLBACK_DATA_LEN || dst.len() < len {
        return Err(CallbackIxError);
    }
    let mut cursor = 0;
    dst[cursor..cursor + CALLBACK_IX_DISCRIMINATOR.len()]
        .copy_from_slice(&CALLBACK_IX_DISCRIMINATOR);
    cursor += CALLBACK_IX_DISCRIMINATOR.len();
    dst[cursor..cursor + CALLBACK_OUTPUT_LEN].copy_from_slice(output);
    cursor += CALLBACK_OUTPUT_LEN;
    let encoded_len = u32::try_from(callback_data.len()).map_err(|_| CallbackIxError)?;
    dst[cursor..cursor + 4].copy_from_slice(&encoded_len.to_le_bytes());
    cursor += 4;
    dst[cursor..cursor + callback_data.len()].copy_from_slice(callback_data);
    Ok(len)
}

#[cfg(test)]
mod tests {
    use super::{write_callback_instruction, CALLBACK_IX_DISCRIMINATOR, CALLBACK_OUTPUT_LEN};

    #[test]
    fn callback_instruction_matches_the_documented_layout() {
        let mut buf = [0u8; CALLBACK_IX_DISCRIMINATOR.len() + CALLBACK_OUTPUT_LEN + 4 + 2];
        let written = write_callback_instruction(&mut buf, &[7u8; CALLBACK_OUTPUT_LEN], &[1, 2])
            .expect("buffer fits");
        assert_eq!(written, buf.len());
        assert_eq!(&buf[..8], &CALLBACK_IX_DISCRIMINATOR);
        assert_eq!(buf[8], 7);
        assert_eq!(&buf[72..76], &2u32.to_le_bytes());
        assert_eq!(&buf[76..], &[1, 2]);
    }
}
