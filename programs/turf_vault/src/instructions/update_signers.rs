use anchor_lang::prelude::*;
use crate::state::VaultState;
use crate::errors::VaultError;

#[derive(Accounts)]
pub struct UpdateSigners<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    pub cosigner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault_state.bump,
        constraint = vault_state.validate_multisig(&admin.key(), &cosigner.key()) @ VaultError::Unauthorized,
    )]
    pub vault_state: Account<'info, VaultState>,
}

pub fn handle_update_signers(ctx: Context<UpdateSigners>, new_signers: [Pubkey; 3], new_threshold: u8) -> Result<()> {
    require!(new_threshold >= 1 && new_threshold <= 3, VaultError::InvalidThreshold);
    require!(new_signers[0] != new_signers[1] && new_signers[0] != new_signers[2] && new_signers[1] != new_signers[2], VaultError::DuplicateSigner);

    let vault = &mut ctx.accounts.vault_state;
    vault.signers = new_signers;
    vault.threshold = new_threshold;

    msg!("Signers updated. New signers: [{}, {}, {}], Threshold: {}", new_signers[0], new_signers[1], new_signers[2], new_threshold);
    Ok(())
}
