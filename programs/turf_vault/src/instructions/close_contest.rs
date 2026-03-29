use anchor_lang::prelude::*;
use crate::state::{VaultState, Contest, ContestStatus};
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct CloseContest<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault"],
        bump = vault_state.bump,
        constraint = vault_state.admin == admin.key() @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        constraint = contest.status == ContestStatus::Settled @ VaultError::ContestNotSettled,
        close = admin,
    )]
    pub contest: Account<'info, Contest>,
}

pub fn handle_close_contest(_ctx: Context<CloseContest>) -> Result<()> {
    msg!("Contest account closed, rent reclaimed");
    Ok(())
}
