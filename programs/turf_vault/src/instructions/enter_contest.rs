use anchor_lang::prelude::*;
use crate::state::{UserAccount, Contest, ContestEntry, ContestStatus, EntryStatus};
use crate::errors::VaultError;

#[derive(Accounts)]
#[instruction(entry_num: u32)]
pub struct EnterContest<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The wallet that owns the user account (may differ from payer for custodial)
    /// CHECK: Validated via user_account PDA seeds
    pub wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"user", wallet.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        mut,
        constraint = contest.status == ContestStatus::Open @ VaultError::ContestNotOpen,
        constraint = contest.current_entries < contest.max_entries @ VaultError::ContestFull,
    )]
    pub contest: Account<'info, Contest>,

    #[account(
        init,
        payer = payer,
        space = 8 + ContestEntry::INIT_SPACE,
        seeds = [
            b"entry",
            contest.contest_id.as_ref(),
            wallet.key().as_ref(),
            &entry_num.to_le_bytes(),
        ],
        bump,
    )]
    pub contest_entry: Account<'info, ContestEntry>,

    pub system_program: Program<'info, System>,
}

pub fn handle_enter_contest(ctx: Context<EnterContest>, entry_num: u32) -> Result<()> {
    let user = &mut ctx.accounts.user_account;
    let contest = &mut ctx.accounts.contest;

    // Debit entry fee from user balance
    require!(user.balance >= contest.entry_fee, VaultError::InsufficientBalance);
    user.balance = user.balance.checked_sub(contest.entry_fee).ok_or(VaultError::Overflow)?;

    // Add to entry fees collected
    contest.entry_fees = contest.entry_fees.checked_add(contest.entry_fee).ok_or(VaultError::Overflow)?;
    contest.current_entries = contest.current_entries.checked_add(1).ok_or(VaultError::Overflow)?;

    // Award 65 seeds
    user.seeds = user.seeds.checked_add(65).ok_or(VaultError::Overflow)?;

    // Create entry
    let entry = &mut ctx.accounts.contest_entry;
    entry.contest_id = contest.contest_id;
    entry.wallet = ctx.accounts.wallet.key();
    entry.entry_num = entry_num;
    entry.status = EntryStatus::Active;
    entry.rank = 0;
    entry.payout = 0;
    entry.bump = ctx.bumps.contest_entry;

    msg!(
        "Entry {} for wallet {} in contest. Entry fees: {}",
        entry_num,
        entry.wallet,
        contest.entry_fees
    );
    Ok(())
}
