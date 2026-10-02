use anchor_lang::prelude::*;

#[error_code]
pub enum OracleRegistryError {
    #[msg("The operator pubkey is the default pubkey")]
    InvalidOperator,
    #[msg("The job mask sets a bit this program does not know")]
    InvalidJobMask,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("The VRF public key is not a valid Ed25519 point")]
    InvalidVrfKey,
    #[msg("Stake parameters are not configured")]
    StakeNotConfigured,
    #[msg("The stake amount is zero")]
    StakeAmountZero,
    #[msg("The node status does not allow this stake change")]
    InvalidStakeStatus,
    #[msg("The node is not unstaking")]
    NotUnstaking,
    #[msg("The unstake cooldown has not elapsed")]
    CooldownNotElapsed,
    #[msg("Only an active node can heartbeat")]
    HeartbeatNotActive,
    #[msg("Slash is not configured")]
    SlashDisabled,
    #[msg("The signer is not the slash authority")]
    InvalidSlashAuthority,
    #[msg("The destination is not the slash destination")]
    InvalidSlashDestination,
    #[msg("The slash amount exceeds the locked stake")]
    SlashExceedsStake,
    #[msg("The node account does not hold the locked stake above rent")]
    InsufficientStake,
    #[msg("The cooldown or heartbeat window is outside the allowed range")]
    InvalidStakeParameter,
    #[msg("Slash authority and destination must both be set or both be empty")]
    InvalidSlashConfig,
    #[msg("The node is not active")]
    NodeNotActive,
}
