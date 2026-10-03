//! executor.data
//!
//! ADR 0004 names source-specific checks and a quorum, and names neither.
//! This process does not fetch arbitrary data.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "executor.data",
    milestone: "unspecified",
};
