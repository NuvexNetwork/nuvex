use anchor_lang::prelude::*;
use nuvex_job_types::mask_is_known;

use crate::errors::OracleRegistryError;
use crate::events::NodeUpdated;

pub fn handler(
    ctx: Context<crate::UpdateNode>,
    operator: Pubkey,
    supported_job_mask: u32,
) -> Result<()> {
    if operator == Pubkey::default() {
        return err!(OracleRegistryError::InvalidOperator);
    }
    if !mask_is_known(supported_job_mask) {
        return err!(OracleRegistryError::InvalidJobMask);
    }
    let node = &mut ctx.accounts.node;
    node.operator = operator;
    node.supported_job_mask = supported_job_mask;
    emit!(NodeUpdated {
        node: node.key(),
        operator,
        supported_job_mask,
    });
    Ok(())
}
