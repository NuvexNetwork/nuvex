use anchor_lang::prelude::*;
use nuvex_job_types::JobType;

use crate::errors::OracleCoreError;
use crate::events::JobConfigured;
use crate::utils::require_timeout;

pub fn handler(
    ctx: Context<crate::ConfigureJob>,
    job_type: u8,
    timeout_slots: u64,
    requests_enabled: bool,
) -> Result<()> {
    require_requestable(job_type)?;
    require_timeout(timeout_slots)?;
    let config = &mut ctx.accounts.job_config;
    config.job_type = job_type;
    config.requests_enabled = requests_enabled;
    config.timeout_slots = timeout_slots;
    config.bump = ctx.bumps.job_config;
    emit!(JobConfigured {
        job_type,
        timeout_slots,
        requests_enabled,
    });
    Ok(())
}

pub(crate) fn require_requestable(job_type: u8) -> Result<()> {
    let job = JobType::from_u8(job_type).ok_or_else(|| error!(OracleCoreError::UnsupportedJob))?;
    if !job.milestone_one_requestable() {
        return err!(OracleCoreError::UnsupportedJob);
    }
    Ok(())
}
