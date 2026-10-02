//! Verification program.
//!
//! `verify_vrf` checks an ECVRF proof with `solana-ecvrf` 0.0.1 and stores the
//! output. Oracle core is the only caller: the instruction requires the core
//! protocol PDA as a signer. Challenge instructions are not implemented.

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

use constants::{PROTOCOL_SEED, VERIFICATION_SEED};
use state::VrfResult;

declare_id!("4f5BaSpKo64bUDEVSkn4h25b8bNspxi24FMfVa4sfurL");

pub const ORACLE_CORE_ID: Pubkey = pubkey!("9CHifhFPRK8nJJpL2nTY17UfYC75aReR4qojVqNaLnoe");

#[program]
pub mod verification {
    use super::*;

    pub fn verify_vrf(
        ctx: Context<VerifyVrf>,
        request: Pubkey,
        node: Pubkey,
        vrf_pubkey: [u8; 32],
        alpha: Vec<u8>,
        proof: [u8; 80],
    ) -> Result<()> {
        instructions::verify_vrf::handler(ctx, request, node, vrf_pubkey, alpha, proof)
    }
}

#[derive(Accounts)]
#[instruction(request: Pubkey)]
pub struct VerifyVrf<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        bump,
        seeds::program = ORACLE_CORE_ID
    )]
    pub protocol: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + VrfResult::INIT_SPACE,
        seeds = [VERIFICATION_SEED, request.as_ref()],
        bump
    )]
    pub result: Account<'info, VrfResult>,
    pub system_program: Program<'info, System>,
}

#[cfg(test)]
mod space_tests {
    use anchor_lang::Space;

    use super::state::VrfResult;

    #[test]
    fn vrf_result_matches_the_field_layout() {
        assert_eq!(VrfResult::INIT_SPACE, 241);
    }
}
