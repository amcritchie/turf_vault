use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Only the vault admin can perform this action")]
    Unauthorized,
    #[msg("Token mint is not accepted by this vault")]
    InvalidMint,
    #[msg("Insufficient balance for this operation")]
    InsufficientBalance,
    #[msg("Contest is not open for entries")]
    ContestNotOpen,
    #[msg("Contest is full")]
    ContestFull,
    #[msg("Contest has not been settled")]
    ContestNotSettled,
    #[msg("Contest is already settled")]
    ContestAlreadySettled,
    #[msg("User already entered this contest with this entry number")]
    DuplicateEntry,
    #[msg("Settlement payouts exceed prize pool plus bonus")]
    SettlementOverflow,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Payout amounts must sum to bonus amount")]
    InvalidPayoutTiers,
}
