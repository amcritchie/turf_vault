use anchor_lang::prelude::*;
use crate::errors::VaultError;

/// Migration-only instruction: closes the old VaultState account so it can be
/// re-initialized with the new schema (which adds admin_backup).
///
/// We cannot use `Account<'info, VaultState>` because the on-chain data has the
/// OLD layout (no admin_backup field). Instead we accept a raw UncheckedAccount,
/// verify the PDA seeds, read the admin pubkey from raw bytes, confirm the
/// signer matches, and then close the account by draining its lamports.
#[derive(Accounts)]
pub struct ForceCloseVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The existing vault account (old layout). We verify PDA seeds manually.
    /// CHECK: Verified via seeds check and manual admin comparison below.
    #[account(
        mut,
        seeds = [b"vault"],
        bump,
    )]
    pub vault_state: UncheckedAccount<'info>,
}

pub fn handle_force_close_vault(ctx: Context<ForceCloseVault>) -> Result<()> {
    let vault_info = &ctx.accounts.vault_state;
    let data = vault_info.try_borrow_data()?;

    // Old layout: 8-byte discriminator + 32-byte admin pubkey
    require!(data.len() >= 40, VaultError::Unauthorized);

    let stored_admin = Pubkey::try_from(&data[8..40])
        .map_err(|_| error!(VaultError::Unauthorized))?;
    require!(stored_admin == ctx.accounts.admin.key(), VaultError::Unauthorized);

    // Drop borrow before modifying lamports
    drop(data);

    // Close the account: drain lamports to admin
    let vault_lamports = vault_info.lamports();
    **vault_info.try_borrow_mut_lamports()? = 0;
    **ctx.accounts.admin.try_borrow_mut_lamports()? = ctx
        .accounts
        .admin
        .lamports()
        .checked_add(vault_lamports)
        .ok_or(VaultError::Overflow)?;

    // Zero out account data so runtime garbage-collects it
    let mut data = vault_info.try_borrow_mut_data()?;
    data.fill(0);

    msg!("Vault account force-closed. Rent reclaimed by admin.");
    Ok(())
}
