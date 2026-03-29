use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::{VaultState, UserAccount};
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(
        seeds = [b"vault"],
        bump = vault_state.bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        constraint = mint.key() == vault_state.usdc_mint || mint.key() == vault_state.usdt_mint @ VaultError::InvalidMint,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = (mint.key() == vault_state.usdc_mint && vault_token_account.key() == vault_state.vault_usdc)
            || (mint.key() == vault_state.usdt_mint && vault_token_account.key() == vault_state.vault_usdt)
            @ VaultError::InvalidMint,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    let user = &mut ctx.accounts.user_account;

    require!(user.balance >= amount, VaultError::InsufficientBalance);

    // Debit user balance
    user.balance = user.balance.checked_sub(amount).ok_or(VaultError::Overflow)?;
    user.total_withdrawn = user.total_withdrawn.checked_add(amount).ok_or(VaultError::Overflow)?;

    // Transfer tokens from vault to user via PDA signer
    let seeds = &[b"vault".as_ref(), &[ctx.accounts.vault_state.bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.vault_token_account.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.vault_state.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    token::transfer(cpi_ctx, amount)?;

    msg!("Withdrew {} from user {}", amount, user.wallet);
    Ok(())
}
