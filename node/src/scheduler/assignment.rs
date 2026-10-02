//! scheduler.assignment
//!
//! Chain fulfillment checks an eligibility predicate. This process does not assign a node.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "scheduler.assignment",
    milestone: "3",
};
