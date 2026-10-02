//! Request transitions.
//!
//! `Allowed` edges have instructions. `Reserved` edges are the multi-step
//! path named in ADR 0001, and no instruction performs them. VRF fulfillment
//! persists `Pending → CallbackExecuted` in one instruction after the proof
//! verifies. Other jobs must not use that edge until their own record says so.
//! Challenge, reject, and fail edges stay forbidden.

use crate::RequestStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transition {
    Allowed,
    Reserved,
    Forbidden,
}

pub const fn transition(from: RequestStatus, to: RequestStatus) -> Transition {
    use RequestStatus::{
        Assigned, CallbackExecuted, Cancelled, Computing, Created, Expired, Finalized, Pending,
        Submitted, Verifying,
    };
    match (from, to) {
        (Pending, Cancelled) | (Pending, Expired) | (Pending, CallbackExecuted) => {
            Transition::Allowed
        }
        (Created, Pending)
        | (Pending, Assigned)
        | (Assigned, Computing)
        | (Assigned, Expired)
        | (Computing, Submitted)
        | (Computing, Expired)
        | (Submitted, Verifying)
        | (Verifying, Finalized)
        | (Finalized, CallbackExecuted) => Transition::Reserved,
        _ => Transition::Forbidden,
    }
}

#[cfg(test)]
mod tests {
    use super::{transition, Transition};
    use crate::RequestStatus;

    const ALL: [RequestStatus; 13] = [
        RequestStatus::Created,
        RequestStatus::Pending,
        RequestStatus::Assigned,
        RequestStatus::Computing,
        RequestStatus::Submitted,
        RequestStatus::Verifying,
        RequestStatus::Finalized,
        RequestStatus::CallbackExecuted,
        RequestStatus::Cancelled,
        RequestStatus::Expired,
        RequestStatus::Rejected,
        RequestStatus::Failed,
        RequestStatus::Challenged,
    ];

    #[test]
    fn cancel_expire_and_vrf_fulfillment_are_allowed() {
        let mut count = 0u8;
        for from in ALL {
            for to in ALL {
                if transition(from, to) == Transition::Allowed {
                    count += 1;
                    assert!(
                        (from, to) == (RequestStatus::Pending, RequestStatus::Cancelled)
                            || (from, to) == (RequestStatus::Pending, RequestStatus::Expired)
                            || (from, to)
                                == (RequestStatus::Pending, RequestStatus::CallbackExecuted)
                    );
                }
            }
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn challenge_edges_are_forbidden() {
        for from in ALL {
            assert_eq!(
                transition(from, RequestStatus::Challenged),
                Transition::Forbidden
            );
            assert_eq!(
                transition(RequestStatus::Challenged, from),
                Transition::Forbidden
            );
        }
    }

    #[test]
    fn terminal_states_do_not_leave() {
        for from in [
            RequestStatus::Cancelled,
            RequestStatus::Expired,
            RequestStatus::Rejected,
            RequestStatus::Failed,
            RequestStatus::CallbackExecuted,
        ] {
            for to in ALL {
                assert_eq!(transition(from, to), Transition::Forbidden);
            }
        }
    }
}
