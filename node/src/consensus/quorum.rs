//! consensus.quorum
//!
//! This subsystem is not implemented. It exports a status record so the process
//! can log that fact. It does not return a job result.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "consensus.quorum",
    milestone: "5",
};
