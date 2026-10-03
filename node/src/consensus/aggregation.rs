//! consensus.aggregation
//!
//! The median of fresh observations is computed in nuvex-services. This process
//! does not aggregate and does not submit the result.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "consensus.aggregation",
    milestone: "5",
};
