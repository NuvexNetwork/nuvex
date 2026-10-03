//! consensus.quorum
//!
//! Price reads name their own minimum source count. There is no on-chain quorum.
//! This process does not vote.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "consensus.quorum",
    milestone: "5",
};
