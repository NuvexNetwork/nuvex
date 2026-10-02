//! proof.vrf
//!
//! The process does not load a VRF secret or submit a transaction.
//! Host proving lives in `nuvex-vrf`. This module only reports status.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "proof.vrf",
    milestone: "2",
};

// Proof generation is absent. See ADR 0004.
