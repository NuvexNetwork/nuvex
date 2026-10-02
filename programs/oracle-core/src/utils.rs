use anchor_lang::prelude::*;
use nuvex_protocol_types::{transition, RequestStatus, Transition};

use crate::constants::{MAX_TIMEOUT_SLOTS, MIN_TIMEOUT_SLOTS};
use crate::errors::OracleCoreError;

pub fn require_timeout(timeout_slots: u64) -> Result<()> {
    if !(MIN_TIMEOUT_SLOTS..=MAX_TIMEOUT_SLOTS).contains(&timeout_slots) {
        return err!(OracleCoreError::InvalidTimeout);
    }
    Ok(())
}

pub fn load_status(status: u8) -> Result<RequestStatus> {
    RequestStatus::from_u8(status).ok_or_else(|| error!(OracleCoreError::UnknownStatus))
}

pub fn require_transition(from: RequestStatus, to: RequestStatus) -> Result<()> {
    match transition(from, to) {
        Transition::Allowed => Ok(()),
        Transition::Reserved => err!(OracleCoreError::TransitionNotEnabled),
        Transition::Forbidden => err!(OracleCoreError::InvalidTransition),
    }
}
