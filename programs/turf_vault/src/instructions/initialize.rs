use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::VaultState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + VaultState::INIT_SPACE,
        seeds = [b"vault"],
        bump,
    )]
    pub vault_state: Account<'info, VaultState>,

    pub usdc_mint: Account<'info, Mint>,
    pub usdt_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        token::mint = usdc_mint,
        token::authority = vault_state,
        seeds = [b"vault_usdc"],
        bump,
    )]
    pub vault_usdc: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        token::mint = usdt_mint,
        token::authority = vault_state,
        seeds = [b"vault_usdt"],
        bump,
    )]
    pub vault_usdt: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    let vault = &mut ctx.accounts.vault_state;
    vault.admin = ctx.accounts.admin.key();
    vault.usdc_mint = ctx.accounts.usdc_mint.key();
    vault.usdt_mint = ctx.accounts.usdt_mint.key();
    vault.vault_usdc = ctx.accounts.vault_usdc.key();
    vault.vault_usdt = ctx.accounts.vault_usdt.key();
    vault.bump = ctx.bumps.vault_state;

    msg!("Vault initialized. Admin: {}", vault.admin);
    Ok(())
}
