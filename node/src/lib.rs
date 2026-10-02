#![forbid(unsafe_code)]
//! Oracle node library.
//!
//! Job execution modules report that they are disabled. None of them produce
//! a result, a price, or a proof.

pub mod config;
pub mod consensus;
pub mod executor;
pub mod network;
pub mod node;
pub mod proof;
pub mod scheduler;
pub mod solana;
pub mod storage;
pub mod telemetry;

pub use node::SubsystemInfo;
