//! executor.ai
//!
//! This subsystem is not implemented. It exports a status record so the process
//! can log that fact. It does not return a job result.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "executor.ai",
    milestone: "8",
};

// Inference is not stubbed. There is no model call in this module.
