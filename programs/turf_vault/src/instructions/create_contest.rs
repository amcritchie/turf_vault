use anchor_lang::prelude::*;
use crate::state::{VaultState, Contest, ContestStatus};
use crate::errors::VaultError;

#[derive(Accounts)]
#[instruction(contest_id: [u8; 32])]
pub struct CreateContest<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"vault"],
        bump = vault_state.bump,
        constraint = vault_state.admin == admin.key() @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = admin,
        space = 8 + Contest::INIT_SPACE,
        seeds = [b"contest", contest_id.as_ref()],
        bump,
    )]
    pub contest: Account<'info, Contest>,

    pub system_program: Program<'info, System>,
}

pub fn handle_create_contest(
    ctx: Context<CreateContest>,
    contest_id: [u8; 32],
    entry_fee: u64,
    max_entries: u32,
    payout_bps: Vec<u16>,
    bonus: u64,
) -> Result<()> {
    // Validate payout_bps sum <= 10000 (100%)
    let total_bps: u32 = payout_bps.iter().map(|&b| b as u32).sum();
    require!(total_bps <= 10_000, VaultError::InvalidPayoutTiers);

    let contest = &mut ctx.accounts.contest;
    contest.contest_id = contest_id;
    contest.entry_fee = entry_fee;
    contest.max_entries = max_entries;
    contest.current_entries = 0;
    contest.prize_pool = 0;
    contest.bonus = bonus;
    contest.status = ContestStatus::Open;
    contest.payout_bps = payout_bps;
    contest.admin = ctx.accounts.admin.key();
    contest.bump = ctx.bumps.contest;

    msg!("Contest created with fee: {}", entry_fee);
    Ok(())
}
