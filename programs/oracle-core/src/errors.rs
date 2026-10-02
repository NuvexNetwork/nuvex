use anchor_lang::prelude::*;

#[error_code]
pub enum OracleCoreError {
    #[msg("The protocol is paused")]
    ProtocolPaused,
    #[msg("This job type cannot be requested")]
    UnsupportedJob,
    #[msg("Job requests are disabled")]
    JobRequestsDisabled,
    #[msg("The request status transition is not allowed")]
    InvalidTransition,
    #[msg("The request status transition is reserved for a later milestone")]
    TransitionNotEnabled,
    #[msg("The request has not expired")]
    NotExpired,
    #[msg("The request has expired")]
    RequestExpired,
    #[msg("The request is not in a terminal state")]
    NotTerminal,
    #[msg("Input exceeds the maximum length")]
    InputTooLarge,
    #[msg("Callback data exceeds the maximum length")]
    CallbackDataTooLarge,
    #[msg("Timeout is outside the allowed range")]
    InvalidTimeout,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("The stored request status is not a known value")]
    UnknownStatus,
    #[msg("Only a VRF request can be fulfilled")]
    RequestNotVrf,
    #[msg("The node cannot fulfill this VRF request")]
    NodeCannotFulfill,
    #[msg("The node registered its VRF key too late")]
    NodeNotEligible,
    #[msg("The callback program account does not match the request")]
    CallbackProgramMismatch,
    #[msg("The VRF alpha could not be bound to the request")]
    AlphaInvalid,
    #[msg("The stored verification account does not match this fulfillment")]
    VerificationMismatch,
    #[msg("Stake parameters are not configured")]
    StakeNotConfigured,
    #[msg("The node is not active")]
    NodeNotActive,
    #[msg("The node stake is below the configured minimum")]
    StakeBelowMinimum,
    #[msg("The node has not sent a heartbeat")]
    HeartbeatMissing,
    #[msg("The node heartbeat is older than the configured window")]
    HeartbeatStale,
}
