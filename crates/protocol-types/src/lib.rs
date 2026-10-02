#![forbid(unsafe_code)]
//! Provisional protocol vocabulary.
//!
//! The request states below are the names from the protocol specification.
//! Their numeric discriminants are the `u8` stored on `OracleRequest`. The enum
//! is not Borsh-serialized, so the discriminant cannot drift with variant order.
//!
//! The legal edges are in [`transition`]. See ADR 0001. Milestone 2 adds
//! `Pending → CallbackExecuted` for a verified VRF fulfillment.

#![no_std]

pub mod transition;

pub use transition::{transition, Transition};

/// Request lifecycle names.
///
/// Discriminants are the on-chain `u8` stored on `OracleRequest`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RequestStatus {
    Created = 1,
    Pending = 2,
    Assigned = 3,
    Computing = 4,
    Submitted = 5,
    Verifying = 6,
    Finalized = 7,
    CallbackExecuted = 8,
    Cancelled = 9,
    Expired = 10,
    Rejected = 11,
    Failed = 12,
    Challenged = 13,
}

impl RequestStatus {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Created),
            2 => Some(Self::Pending),
            3 => Some(Self::Assigned),
            4 => Some(Self::Computing),
            5 => Some(Self::Submitted),
            6 => Some(Self::Verifying),
            7 => Some(Self::Finalized),
            8 => Some(Self::CallbackExecuted),
            9 => Some(Self::Cancelled),
            10 => Some(Self::Expired),
            11 => Some(Self::Rejected),
            12 => Some(Self::Failed),
            13 => Some(Self::Challenged),
            _ => None,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::CallbackExecuted
                | Self::Cancelled
                | Self::Expired
                | Self::Rejected
                | Self::Failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::RequestStatus;

    #[test]
    fn provisional_discriminants_round_trip() {
        for value in 1u8..=13 {
            let status = RequestStatus::from_u8(value).expect("known status");
            assert_eq!(status.as_u8(), value);
        }
    }

    #[test]
    fn unknown_status_is_rejected() {
        assert!(RequestStatus::from_u8(0).is_none());
        assert!(RequestStatus::from_u8(14).is_none());
        assert!(RequestStatus::from_u8(255).is_none());
    }

    #[test]
    fn challenged_is_not_marked_terminal() {
        // Whether a challenge can return to finalization is undecided.
        // Treating it as terminal would be a security assumption.
        assert!(!RequestStatus::Challenged.is_terminal());
    }
}
