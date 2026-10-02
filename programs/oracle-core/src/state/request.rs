use anchor_lang::prelude::*;

use crate::constants::{MAX_CALLBACK_DATA_LEN, MAX_INPUT_LEN, REQUEST_ID_LEN};

#[account]
#[derive(InitSpace)]
pub struct OracleRequest {
    pub requester: Pubkey,
    pub job_type: u8,
    pub status: u8,
    /// Recorded ceiling. No instruction transfers this amount.
    pub max_fee: u64,
    pub created_slot: u64,
    pub expires_slot: u64,
    pub request_id: [u8; REQUEST_ID_LEN],
    pub input_len: u16,
    pub input: [u8; MAX_INPUT_LEN],
    pub callback_program: Pubkey,
    pub callback_data_len: u16,
    pub callback_data: [u8; MAX_CALLBACK_DATA_LEN],
    pub bump: u8,
    /// Default until fulfill. The node that met the eligibility predicate.
    pub assigned_node: Pubkey,
    /// Stake read at fulfill. Not a later balance.
    pub assigned_stake: u64,
    /// Heartbeat slot read at fulfill.
    pub assigned_heartbeat: u64,
}
