use anchor_lang::prelude::*;

use crate::errors::OracleCoreError;
use crate::events::JobConfigured;
use crate::instructions::configure_job::require_requestable;
use crate::utils::require_timeout;

pub fn handler(
    ctx: Context<crate::UpdateJob>,
    job_type: u8,
    timeout_slots: u64,
    requests_enabled: bool,
) -> Result<()> {
    require_requestable(job_type)?;
    require_timeout(timeout_slots)?;
    let config = &mut ctx.accounts.job_config;
    if config.job_type != job_type {
        return err!(OracleCoreError::UnsupportedJob);
    }
    config.requests_enabled = requests_enabled;
    config.timeout_slots = timeout_slots;
    emit!(JobConfigured {
        job_type,
        timeout_slots,
        requests_enabled,
    });
    Ok(())
}
