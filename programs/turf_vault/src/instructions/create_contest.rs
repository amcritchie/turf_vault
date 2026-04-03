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
        constraint = vault_state.is_admin(&admin.key()) @ VaultError::Unauthorized,
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
    payout_amounts: Vec<u64>,
    bonus: u64,
) -> Result<()> {
    // Validate payout_amounts sum == bonus
    let total_payouts: u64 = payout_amounts.iter().sum();
    require!(total_payouts == bonus, VaultError::InvalidPayoutTiers);

    let contest = &mut ctx.accounts.contest;
    contest.contest_id = contest_id;
    contest.entry_fee = entry_fee;
    contest.max_entries = max_entries;
    contest.current_entries = 0;
    contest.prize_pool = 0;
    contest.bonus = bonus;
    contest.status = ContestStatus::Open;
    contest.payout_amounts = payout_amounts;
    contest.admin = ctx.accounts.admin.key();
    contest.bump = ctx.bumps.contest;

    msg!("Contest created with fee: {}", entry_fee);
    Ok(())
}
