#![forbid(unsafe_code)]
//! Job identifiers.
//!
//! A job type names a unit of work. It does not execute that work. Milestone 1
//! can open a VRF request and cannot fulfill it. `AiInference` has no executor
//! anywhere in this repo.

#![no_std]

/// Closed set of job kinds the protocol intends to support.
///
/// Discriminants are the on-chain `u8` stored on job and request accounts.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JobType {
    Vrf = 1,
    Price = 2,
    Data = 3,
    Compute = 4,
    AiInference = 5,
}

impl JobType {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Vrf),
            2 => Some(Self::Price),
            3 => Some(Self::Data),
            4 => Some(Self::Compute),
            5 => Some(Self::AiInference),
            _ => None,
        }
    }

    /// Bit in a `u32` capability mask. Bit 0 is reserved and unused.
    pub const fn capability_bit(self) -> u32 {
        1_u32 << (self as u8)
    }

    pub const fn milestone(self) -> u8 {
        match self {
            Self::Vrf => 2,
            Self::Price | Self::Data => 5,
            Self::Compute => 6,
            Self::AiInference => 8,
        }
    }

    /// Milestone 1 can open a request for this job. It still cannot fulfill it.
    pub const fn milestone_one_requestable(self) -> bool {
        matches!(self, Self::Vrf)
    }
}

pub const fn known_job_mask() -> u32 {
    JobType::Vrf.capability_bit()
        | JobType::Price.capability_bit()
        | JobType::Data.capability_bit()
        | JobType::Compute.capability_bit()
        | JobType::AiInference.capability_bit()
}

pub const fn mask_is_known(mask: u32) -> bool {
    mask & !known_job_mask() == 0
}

#[cfg(test)]
mod tests {
    use super::{known_job_mask, mask_is_known, JobType};

    #[test]
    fn discriminants_round_trip() {
        for value in 1u8..=5 {
            let job = JobType::from_u8(value).expect("known job");
            assert_eq!(job.as_u8(), value);
        }
    }

    #[test]
    fn unknown_job_is_rejected() {
        assert!(JobType::from_u8(0).is_none());
        assert!(JobType::from_u8(6).is_none());
    }

    #[test]
    fn only_vrf_can_be_requested_in_milestone_one() {
        assert!(JobType::Vrf.milestone_one_requestable());
        assert!(!JobType::Price.milestone_one_requestable());
        assert!(!JobType::Data.milestone_one_requestable());
        assert!(!JobType::Compute.milestone_one_requestable());
        assert!(!JobType::AiInference.milestone_one_requestable());
    }

    #[test]
    fn unknown_mask_bits_are_rejected() {
        assert!(mask_is_known(0));
        assert!(mask_is_known(known_job_mask()));
        assert!(!mask_is_known(1));
        assert!(!mask_is_known(known_job_mask() | 1));
    }

    #[test]
    fn capability_bits_do_not_overlap() {
        let mut seen = 0_u32;
        for value in 1u8..=5 {
            let bit = JobType::from_u8(value).expect("known job").capability_bit();
            assert_eq!(seen & bit, 0);
            seen |= bit;
        }
    }
}
