# TurfVault — Development Instructions

## Project Overview

Anchor smart contract for contest escrow on Solana. Backend for Turf Monster (Rails pick'em app).

- **Program ID**: `7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J`
- **Framework**: Anchor 0.32.1
- **Rust**: 1.89.0 (via `rust-toolchain.toml`)
- **Network**: Localnet (dev), Devnet (staging)
- **Version**: 0.7.0

## File Layout

```
programs/turf_vault/src/
├── lib.rs              # Program entry — 11 instruction handlers (thin wrappers)
├── state.rs            # 4 account structs + 2 enums + VaultState::is_admin()
├── errors.rs           # 11 error codes (VaultError enum)
└── instructions/
    ├── mod.rs           # Re-exports all instruction modules
    ├── initialize.rs    # Vault setup, token account init, accepts admin_backup
    ├── create_user_account.rs
    ├── deposit.rs       # User → vault token transfer via CPI
    ├── withdraw.rs      # Vault → user token transfer via PDA signer
    ├── create_contest.rs
    ├── enter_contest.rs # Debit PDA balance, build prize pool (managed wallets)
    ├── enter_contest_direct.rs # User signs USDC transfer from wallet ATA (Phantom wallets)
    ├── settle_contest.rs # remaining_accounts pattern, manual PDA verify
    ├── close_contest.rs
    └── force_close_vault.rs # Migration-only: closes old vault for re-init
tests/
└── turf_vault.ts       # 20 test cases covering all instructions + dual admin
Anchor.toml             # Program ID, cluster config, test script
```

## Dual Admin System

VaultState stores both `admin` (primary) and `admin_backup` pubkeys. The `is_admin()` helper checks both:

```rust
pub fn is_admin(&self, key: &Pubkey) -> bool {
    self.admin == *key || self.admin_backup == *key
}
```

- **Primary admin (Alex Bot)**: `F6f8h5yynbnkgWvU5abQx3RJxJpe8EoQmeFBuNKdKzhZ` — used for all operations (create, lock, settle)
- **Backup admin (Alex Human)**: `7ZDJp7FUHhuceAqcW9CHe81hCiaMTjgWAXfprBM59Tcr` — recovery key only
- `initialize` accepts `admin_backup: Pubkey` argument
- `create_contest`, `settle_contest`, `close_contest` all use `vault_state.is_admin()` constraint
- `force_close_vault` reads admin from raw bytes (for migrating old-schema vaults)

## Anchor Patterns

### PDA Derivation

| Account | Seeds | Notes |
|---------|-------|-------|
| VaultState | `[b"vault"]` | Singleton |
| UserAccount | `[b"user", wallet]` | One per wallet |
| Contest | `[b"contest", contest_id]` | contest_id = SHA256 of Rails slug |
| ContestEntry | `[b"entry", contest_id, wallet, entry_num.to_le_bytes()]` | Multiple per user |

### CPI Token Transfers

- **Deposit** (user → vault): Standard CPI `transfer` with user as signer
- **Withdraw** (vault → user): CPI `transfer` with PDA seeds `[b"vault", &[bump]]` as signer

### remaining_accounts Pattern (settle_contest)

Settlement passes entry data as `remaining_accounts` — pairs of `[user_account, contest_entry]`. Each pair is:
1. PDA-verified against expected seeds
2. Manually deserialized (`try_deserialize`)
3. Mutated (balance, rank, payout, status)
4. Manually serialized back (`try_serialize`)

This pattern avoids Anchor's account resolution limits for variable-length settlement arrays.

### Account Sizing

All accounts use `#[derive(InitSpace)]`. Contest has `#[max_len(10)]` on `payout_amounts: Vec<u64>` (max 10 payout tiers).

## State Model

### Enums
- `ContestStatus`: Open → Locked → Settled
- `EntryStatus`: Active → Won / Lost

### UserAccount Fields
- `wallet`, `balance`, `total_deposited`, `total_withdrawn`, `total_won`, `seeds` (u64, 65 per entry), `bump`

### Contest Fields
- `contest_id`, `entry_fee`, `max_entries`, `current_entries`, `prize_pool`, `bonus`, `status`, `payout_amounts` (Vec, max 10), `admin` (payer pubkey), `creator` (bonus funder pubkey), `bump`

### Key Constraints
- All token amounts: `u64` with 6 decimals (1 USDC = 1_000_000)
- `payout_amounts` sum must equal `bonus` (validated in create_contest)
- Settlement total payouts must be ≤ prize_pool + bonus
- All arithmetic uses `checked_add`/`checked_sub`

## Error Codes

| Code | Name | When |
|------|------|------|
| 6000 | Unauthorized | Non-admin tries admin action |
| 6001 | InvalidMint | Deposit/withdraw with wrong mint |
| 6002 | InsufficientBalance | Withdraw/enter exceeds balance |
| 6003 | ContestNotOpen | Enter non-open contest |
| 6004 | ContestFull | Contest at max_entries |
| 6005 | ContestNotSettled | Close unsettled contest |
| 6006 | ContestAlreadySettled | Settle already-settled contest |
| 6007 | DuplicateEntry | Same entry_num (PDA collision) |
| 6008 | SettlementOverflow | Payouts > pool + bonus |
| 6009 | Overflow | Arithmetic overflow |
| 6010 | InvalidPayoutTiers | Bad payout_amounts config |

## Testing

### Run Tests
```bash
anchor test
```

20 tests covering: initialize (with admin_backup), create_user_account, deposit (USDC/USDT + invalid mint), create_contest (admin with bonus USDC transfer + non-admin rejection), enter_contest (2 users + insufficient balance), settle_contest (payouts + already-settled + non-admin), withdraw (success + insufficient balance), close_contest (settled + unsettled), backup admin (can create contest). Tests use SOL transfers from admin instead of `requestAirdrop` (broken in Solana v3.1).

### Test Setup Pattern
```typescript
// Initialize with backup admin
await program.methods
  .initialize(adminBackup.publicKey)
  .accountsStrict({ ... })
  .rpc();

// Verify both admin fields
const vault = await program.account.vaultState.fetch(vaultStatePda);
expect(vault.admin.toBase58()).to.equal(admin.publicKey.toBase58());
expect(vault.adminBackup.toBase58()).to.equal(adminBackup.publicKey.toBase58());
```

## Prerequisites

- **Rust**: 1.89.0 (via `rust-toolchain.toml`)
- **Anchor CLI**: 0.32.1 — `/Users/alex/.cargo/bin/anchor`
- **Solana CLI**: `/Users/alex/.local/share/solana/install/active_release/bin/solana`
- **Node.js + Yarn**: Required for TypeScript tests

## Build & Deploy

```bash
# Build
anchor build

# Test (starts local validator automatically)
anchor test

# Deploy to devnet
solana config set --url devnet
anchor deploy --provider.cluster devnet

# Verify deployment
solana program show 7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J
```

### `anchor test` Workaround

Anchor CLI 0.32.1 can't find `node`/`yarn` in PATH due to Rust subprocess spawning. If `anchor test` fails to find node, deploy manually then run tests directly:

```bash
ANCHOR_PROVIDER_URL=http://127.0.0.1:8899 ANCHOR_WALLET=~/.config/solana/id.json yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts
```

### Devnet SOL Faucet Protocol

Follow this sequence when SOL is needed. Move to the next step only if the current one fails.

| Step | Method | Command / URL | Notes |
|------|--------|---------------|-------|
| 1 | **PoW faucet** | `devnet-pow mine --target-lamports <amount> -ud` | Preferred. Consistent, no rate limits. Install: `cargo install devnet-pow`. If public RPC times out, pass `-u <rpc_url>` with a provider endpoint. |
| 2 | **QuickNode faucet** | https://faucet.quicknode.com/solana/devnet | Web UI, no account required. Paste wallet address. |
| 3 | **Solana Foundation faucet** | https://faucet.solana.com | Web UI. Select Devnet, paste address. |
| 4 | **CLI airdrop** | `solana airdrop <amount> --url devnet` | Last resort — frequently rate-limited. Try smaller amounts (0.5 SOL). |
| 5 | **Transfer from funded wallet** | `solana transfer <to> <amount> --url devnet` | If another project wallet has spare SOL. |

**Never rely solely on `solana airdrop`** — it rate-limits aggressively and fails silently under devnet load.

### Migration (Re-initialization)

When the VaultState schema changes (e.g. adding `admin_backup`), the old account must be closed first:

```bash
# From Rails app:
bin/rails solana:init_vault FORCE_CLOSE=true
bin/rails solana:init_vault INIT=true ADMIN_BACKUP=<backup_admin_base58>
```

### Current Deployment (Devnet)

- **Program ID**: `7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J`
- **Vault PDA**: `7z313HTVNcxhvCBkkDQv794RpXeRrfCLb5WJ4dFAQQeh`
- **Admin (primary)**: Alex Bot — `F6f8h5yynbnkgWvU5abQx3RJxJpe8EoQmeFBuNKdKzhZ`
- **Admin (backup)**: Alex Human — `7ZDJp7FUHhuceAqcW9CHe81hCiaMTjgWAXfprBM59Tcr`
- **USDC Mint**: `222Dcu2RgAXE3T8A4mGSG3kQyXaNjqePx7vva1RdWBN9` (test, 6 decimals)
- **USDT Mint**: `9mxkN8KaVA8FFgDE2LEsn2UbYLPG8Xg9bf4V9MYYi8Ne` (test, 6 decimals)
- **IDL Account**: `DCP2XRu8ZwzsCpXBgu5xa4vTYdYQhKUZRU49iJuFv8Lf`

**Status**: v0.7.0 deployed on devnet. Seeds field added to UserAccount (65 per entry since v0.7.0). Vault re-initialized. Mint authorities (USDC + USDT) transferred to Alex Bot.

## Versioning Protocol

- **Semantic versioning** in `programs/turf_vault/Cargo.toml`
  - MAJOR: Breaking account layout changes
  - MINOR: New instructions or features
  - PATCH: Bug fixes, validation improvements
- **After each deploy**:
  1. Bump version in Cargo.toml
  2. Update CHANGELOG.md
  3. Commit: `git commit -m "v0.X.Y: description"`
  4. Tag: `git tag -a v0.X.Y -m "description"`
  5. Push: `git push origin main --tags`

## Integration with Turf Monster

The Rails app calls TurfVault through a `Solana::Vault` service layer:

- **Contest ID**: SHA256 hash of the contest slug (e.g. `"turf-totals-v1-matchday-1"` → 32-byte array)
- **Entry num**: Sequential integer per user per contest
- **Settlement**: Rails grades contest → builds settlement array → calls settle_contest
- **Token amounts**: Rails stores cents (integer), Solana stores 6-decimal u64. Convert: `amount_cents * 10_000` (cents → 6 decimals)
- **Admin key**: `SOLANA_ADMIN_KEY` env var (base58 private key of Alex Bot)

## Key Design Decisions

- **Managed entry** (`enter_contest`): Separates payer (admin signer) from wallet (entry owner) — deducts from UserAccount PDA balance. For server-managed wallets.
- **Direct entry** (`enter_contest_direct`): User signs USDC transfer from their own wallet ATA to vault. Admin pays PDA rent so user only spends USDC. For Phantom wallets. Added in v0.3.0. Requires `user_account` PDA (for seeds award) since v0.5.0.
- **Hard escrow contest creation** (`create_contest` v0.4.0): Dual-signer — `payer` (admin bot, pays SOL rent) + `creator` (Phantom wallet, signs bonus USDC transfer from creator ATA → vault). Contest struct stores `creator` pubkey. If bonus is 0, no transfer occurs but token accounts are still required.
- **No lock instruction**: Contest can go directly from Open to Settled (Locked status exists but no instruction sets it yet)
- **Dual admin**: Primary admin for operations, backup admin for recovery. Both can perform any admin action.
- **Dual mint**: USDC + USDT supported from day one, separate vault token accounts
- **Seeds system** (v0.5.0, updated v0.7.0): Both `enter_contest` and `enter_contest_direct` award 65 seeds to the user's `UserAccount` PDA. Seeds are on-chain only — Rails reads them via `sync_balance` and derives levels in the UI (`level = seeds / 100 + 1`).
- **Manual settlement**: No on-chain scoring — Rails computes results, admin submits final rankings
- **force_close_vault**: Migration instruction that reads admin from raw bytes (avoids deserialization of old schema)

## Code Style

- Keep instructions in separate files under `instructions/`
- Thin wrappers in `lib.rs`, logic in instruction handlers
- All state in `state.rs`, all errors in `errors.rs`
- Use `msg!()` for on-chain logging at key events
- Prefer `require!()` macro over manual `if/return Err`
