//! Node registry program.
//!
//! Milestone 3 locks lamports as stake, records heartbeats, and counts
//! fulfillments. Oracle core calls `note_fulfillment`. Fees do not move.

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

use constants::{NODE_SEED, PROTOCOL_SEED, REGISTRY_SEED};
use state::{NodeAccount, NodeRegistry};

/// Oracle core program id. This crate must not depend on oracle-core.
pub const ORACLE_CORE_ID: Pubkey = pubkey!("9CHifhFPRK8nJJpL2nTY17UfYC75aReR4qojVqNaLnoe");

declare_id!("8NuitXiyMGv6jE7WQq8mWaqGDyrDDRkgaNhroNnXkPXZ");

#[program]
pub mod oracle_registry {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn register_node(
        ctx: Context<RegisterNode>,
        operator: Pubkey,
        vrf_pubkey: [u8; 32],
        supported_job_mask: u32,
    ) -> Result<()> {
        instructions::register_node::handler(ctx, operator, vrf_pubkey, supported_job_mask)
    }

    pub fn update_node(
        ctx: Context<UpdateNode>,
        operator: Pubkey,
        supported_job_mask: u32,
    ) -> Result<()> {
        instructions::update_node::handler(ctx, operator, supported_job_mask)
    }

    pub fn configure_stake(
        ctx: Context<ConfigureStake>,
        min_stake: u64,
        unstake_cooldown_slots: u64,
        heartbeat_timeout_slots: u64,
        slash_authority: Pubkey,
        slash_destination: Pubkey,
    ) -> Result<()> {
        instructions::configure::handler(
            ctx,
            min_stake,
            unstake_cooldown_slots,
            heartbeat_timeout_slots,
            slash_authority,
            slash_destination,
        )
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        instructions::stake::handler(ctx, amount)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        instructions::unstake::handler(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw::handler(ctx)
    }

    pub fn heartbeat(ctx: Context<Heartbeat>) -> Result<()> {
        instructions::heartbeat::handler(ctx)
    }

    pub fn slash(ctx: Context<Slash>, amount: u64) -> Result<()> {
        instructions::slash::handler(ctx, amount)
    }

    pub fn note_fulfillment(ctx: Context<NoteFulfillment>) -> Result<()> {
        instructions::note_fulfillment::handler(ctx)
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + NodeRegistry::INIT_SPACE,
        seeds = [REGISTRY_SEED],
        bump
    )]
    pub registry: Account<'info, NodeRegistry>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterNode<'info> {
    #[account(mut)]
    pub node_authority: Signer<'info>,
    #[account(mut, seeds = [REGISTRY_SEED], bump = registry.bump)]
    pub registry: Account<'info, NodeRegistry>,
    #[account(
        init,
        payer = node_authority,
        space = 8 + NodeAccount::INIT_SPACE,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump
    )]
    pub node: Account<'info, NodeAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateNode<'info> {
    pub node_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump = node.bump,
        has_one = node_authority
    )]
    pub node: Account<'info, NodeAccount>,
}

#[derive(Accounts)]
pub struct ConfigureStake<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [REGISTRY_SEED],
        bump = registry.bump,
        has_one = authority
    )]
    pub registry: Account<'info, NodeRegistry>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub node_authority: Signer<'info>,
    #[account(seeds = [REGISTRY_SEED], bump = registry.bump)]
    pub registry: Account<'info, NodeRegistry>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump = node.bump,
        has_one = node_authority
    )]
    pub node: Account<'info, NodeAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    pub node_authority: Signer<'info>,
    #[account(seeds = [REGISTRY_SEED], bump = registry.bump)]
    pub registry: Account<'info, NodeRegistry>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump = node.bump,
        has_one = node_authority
    )]
    pub node: Account<'info, NodeAccount>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub node_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump = node.bump,
        has_one = node_authority
    )]
    pub node: Account<'info, NodeAccount>,
}

#[derive(Accounts)]
pub struct Heartbeat<'info> {
    pub node_authority: Signer<'info>,
    #[account(
        mut,
        seeds = [NODE_SEED, node_authority.key().as_ref()],
        bump = node.bump,
        has_one = node_authority
    )]
    pub node: Account<'info, NodeAccount>,
}

#[derive(Accounts)]
pub struct Slash<'info> {
    pub slash_authority: Signer<'info>,
    #[account(seeds = [REGISTRY_SEED], bump = registry.bump)]
    pub registry: Account<'info, NodeRegistry>,
    #[account(mut)]
    pub node: Account<'info, NodeAccount>,
    /// CHECK: receives slashed lamports. The key must match `slash_destination`.
    #[account(mut)]
    pub destination: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct NoteFulfillment<'info> {
    #[account(
        seeds = [PROTOCOL_SEED],
        seeds::program = ORACLE_CORE_ID,
        bump
    )]
    pub protocol: Signer<'info>,
    #[account(mut)]
    pub node: Account<'info, NodeAccount>,
}

#[cfg(test)]
mod space_tests {
    use anchor_lang::Space;

    use super::state::NodeAccount;
    use super::state::NodeRegistry;

    #[test]
    fn account_spaces_match_the_field_layout() {
        assert_eq!(NodeRegistry::INIT_SPACE, 129);
        assert_eq!(NodeAccount::INIT_SPACE, 142);
    }
}
