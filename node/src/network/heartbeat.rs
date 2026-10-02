//! network.heartbeat
//!
//! The chain heartbeat instruction exists. This process does not send it.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "network.heartbeat",
    milestone: "3",
};
