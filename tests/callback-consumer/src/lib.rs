//! Test consumer for the Milestone 2 callback layout.
//!
//! The program id is fixed so LiteSVM can load the binary. No keypair is
//! checked in. This crate is not a protocol program.

use anchor_lang::prelude::*;

declare_id!("F9gDNxRu11PBDiRHxqA6p68CLMBQJeMJrsEhvMKFWYy1");

#[program]
pub mod callback_consumer {
    use super::*;

    pub fn callback(_ctx: Context<Callback>, output: [u8; 64], data: Vec<u8>) -> Result<()> {
        if data.first() == Some(&0xff) {
            return err!(CallbackConsumerError::Rejected);
        }
        msg!("nuvex-callback {}", output[0]);
        let _ = data.len();
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Callback {}

#[error_code]
pub enum CallbackConsumerError {
    #[msg("callback rejected")]
    Rejected,
}

#[cfg(test)]
mod tests {
    use anchor_lang::InstructionData;

    use nuvex_common::{
        write_callback_instruction, CALLBACK_IX_DISCRIMINATOR, CALLBACK_OUTPUT_LEN,
    };

    #[test]
    fn anchor_callback_matches_the_shared_layout() {
        let output = [4u8; 64];
        let data = vec![9u8, 8];
        let anchor_data = super::instruction::Callback {
            output,
            data: data.clone(),
        }
        .data();
        let mut written = [0u8; 8 + CALLBACK_OUTPUT_LEN + 4 + 2];
        let len = write_callback_instruction(&mut written, &output, &data).expect("layout");
        assert_eq!(anchor_data, written[..len]);
        assert_eq!(&anchor_data[..8], &CALLBACK_IX_DISCRIMINATOR);
    }
}
