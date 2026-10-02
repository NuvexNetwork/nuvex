//! Registry PDA seeds and node status values.

pub use nuvex_common::{NODE_SEED, PROTOCOL_SEED, REGISTRY_SEED};

/// Created, with no claim on fulfillment.
pub const NODE_STATUS_REGISTERED: u8 = 1;
/// Stake is at least the configured minimum. Heartbeat and fulfill are allowed.
pub const NODE_STATUS_ACTIVE: u8 = 2;
/// The full stake is locked until `cooldown_end_slot`.
pub const NODE_STATUS_UNSTAKING: u8 = 3;
