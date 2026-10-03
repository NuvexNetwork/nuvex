//! executor.price
//!
//! Public price aggregation lives in nuvex-services. This process does not
//! fetch those venues and does not submit a price transaction.

use crate::node::SubsystemInfo;

pub const STATUS: SubsystemInfo = SubsystemInfo {
    name: "executor.price",
    milestone: "5",
};
