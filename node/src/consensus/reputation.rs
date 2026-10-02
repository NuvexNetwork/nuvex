//! consensus.reputation
//!
//! Reputation on the node account is a fulfillment count. This process does not update it.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "consensus.reputation",
    milestone: "3",
};
