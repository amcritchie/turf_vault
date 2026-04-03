use anchor_lang::prelude::*;

pub mod errors;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J");

#[program]
pub mod turf_vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        handle_initialize(ctx)
    }

    pub fn create_user_account(ctx: Context<CreateUserAccount>, wallet: Pubkey) -> Result<()> {
        handle_create_user_account(ctx, wallet)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        handle_deposit(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        handle_withdraw(ctx, amount)
    }

    pub fn create_contest(
        ctx: Context<CreateContest>,
        contest_id: [u8; 32],
        entry_fee: u64,
        max_entries: u32,
        payout_amounts: Vec<u64>,
        bonus: u64,
    ) -> Result<()> {
        handle_create_contest(ctx, contest_id, entry_fee, max_entries, payout_amounts, bonus)
    }

    pub fn enter_contest(ctx: Context<EnterContest>, entry_num: u32) -> Result<()> {
        handle_enter_contest(ctx, entry_num)
    }

    pub fn settle_contest(ctx: Context<SettleContest>, settlements: Vec<Settlement>) -> Result<()> {
        handle_settle_contest(ctx, settlements)
    }

    pub fn close_contest(ctx: Context<CloseContest>) -> Result<()> {
        handle_close_contest(ctx)
    }
}
