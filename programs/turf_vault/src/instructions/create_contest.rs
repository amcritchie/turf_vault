use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::{VaultState, Contest, ContestStatus};
use crate::errors::VaultError;

#[derive(Accounts)]
#[instruction(contest_id: [u8; 32])]
pub struct CreateContest<'info> {
    /// Admin bot — pays SOL rent for Contest PDA
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Creator's Phantom wallet — signs USDC transfer for bonus
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        seeds = [b"vault"],
        bump = vault_state.bump,
        constraint = vault_state.is_admin(&payer.key()) @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        init,
        payer = payer,
        space = 8 + Contest::INIT_SPACE,
        seeds = [b"contest", contest_id.as_ref()],
        bump,
    )]
    pub contest: Account<'info, Contest>,

    #[account(constraint = mint.key() == vault_state.usdc_mint @ VaultError::InvalidMint)]
    pub mint: Account<'info, Mint>,

    /// Creator's USDC token account (ATA) — source of bonus
    #[account(mut, token::mint = mint, token::authority = creator)]
    pub creator_token_account: Account<'info, TokenAccount>,

    /// Vault's USDC token account — destination for bonus
    #[account(mut, constraint = vault_token_account.key() == vault_state.vault_usdc @ VaultError::InvalidMint)]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
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
    contest.admin = ctx.accounts.payer.key();
    contest.creator = ctx.accounts.creator.key();
    contest.bump = ctx.bumps.contest;

    // Transfer bonus USDC from creator's ATA → vault
    if bonus > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.creator.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, bonus)?;
    }

    msg!("Contest created with fee: {}, bonus: {}", entry_fee, bonus);
    Ok(())
}
