use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct JobConfig {
    pub job_type: u8,
    pub requests_enabled: bool,
    pub timeout_slots: u64,
    pub bump: u8,
}
