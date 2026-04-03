# Changelog

All notable changes to TurfVault are documented here. Format based on [Keep a Changelog](https://keepachangelog.com/).

## [0.1.0] - 2026-04-02

### Added
- Initial Anchor program with 8 instructions: initialize, create_user_account, deposit, withdraw, create_contest, enter_contest, settle_contest, close_contest
- VaultState, UserAccount, Contest, ContestEntry account structures
- USDC + USDT dual-mint deposit/withdraw support (6 decimals)
- Contest lifecycle: create → enter → settle → close
- Payout basis points system (up to 10 ranks, sum ≤ 10000 bps)
- Admin-funded bonus pool on top of entry fees
- Settlement via remaining_accounts with PDA verification
- Checked arithmetic on all balance operations
- Full TypeScript test suite (19 tests covering all instructions + error cases)
- Deployed to devnet: `7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J`
