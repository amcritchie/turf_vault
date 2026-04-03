use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub admin: Pubkey,
    pub usdc_mint: Pubkey,
    pub usdt_mint: Pubkey,
    pub vault_usdc: Pubkey,
    pub vault_usdt: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserAccount {
    pub wallet: Pubkey,
    pub balance: u64,          // 6 decimals (USDC/USDT native)
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub total_won: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ContestStatus {
    Open,
    Locked,
    Settled,
}

#[account]
#[derive(InitSpace)]
pub struct Contest {
    pub contest_id: [u8; 32],  // SHA256 of Rails slug
    pub entry_fee: u64,        // 6 decimals
    pub max_entries: u32,
    pub current_entries: u32,
    pub prize_pool: u64,
    pub bonus: u64,            // admin-funded bonus on top of pool
    pub status: ContestStatus,
    #[max_len(10)]
    pub payout_amounts: Vec<u64>,  // USDC amounts per rank (6 decimals, e.g. [40_000000])
    pub admin: Pubkey,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum EntryStatus {
    Active,
    Won,
    Lost,
}

#[account]
#[derive(InitSpace)]
pub struct ContestEntry {
    pub contest_id: [u8; 32],
    pub wallet: Pubkey,
    pub entry_num: u32,
    pub status: EntryStatus,
    pub rank: u32,
    pub payout: u64,
    pub bump: u8,
}
