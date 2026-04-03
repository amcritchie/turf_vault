# TurfVault — Development Instructions

## Project Overview

Anchor smart contract for contest escrow on Solana. Backend for Turf Monster (Rails pick'em app).

- **Program ID**: `7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J`
- **Framework**: Anchor 0.32.1
- **Rust**: 1.89.0 (via `rust-toolchain.toml`)
- **Network**: Localnet (dev), Devnet (staging)
- **Admin keypair**: `~/.config/solana/id.json`

## File Layout

```
programs/turf_vault/src/
├── lib.rs              # Program entry — 8 instruction handlers (thin wrappers)
├── state.rs            # 4 account structs + 2 enums
├── errors.rs           # 11 error codes (VaultError enum)
└── instructions/
    ├── mod.rs           # Re-exports all instruction modules
    ├── initialize.rs    # Vault setup, token account init
    ├── create_user_account.rs
    ├── deposit.rs       # User → vault token transfer via CPI
    ├── withdraw.rs      # Vault → user token transfer via PDA signer
    ├── create_contest.rs
    ├── enter_contest.rs # Debit balance, build prize pool
    ├── settle_contest.rs # remaining_accounts pattern, manual PDA verify
    └── close_contest.rs
tests/
└── turf_vault.ts       # 19 test cases covering all instructions + errors
Anchor.toml             # Program ID, cluster config, test script
```

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

All accounts use `#[derive(InitSpace)]`. Contest has `#[max_len(10)]` on `payout_bps: Vec<u16>` (max 10 payout tiers).

## State Model

### Enums
- `ContestStatus`: Open → Locked → Settled
- `EntryStatus`: Active → Won / Lost

### Key Constraints
- All token amounts: `u64` with 6 decimals (1 USDC = 1_000_000)
- `payout_bps` sum must be ≤ 10,000 (100%)
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
| 6010 | InvalidPayoutTiers | Bad payout_bps config |

## Testing

### Run Tests
```bash
anchor test
```

### Test Setup Pattern
```typescript
// Create mint with 6 decimals
const usdcMint = await createMint(provider.connection, admin, admin.publicKey, null, 6);

// Derive PDAs
const [vaultPda] = PublicKey.findProgramAddressSync([Buffer.from("vault")], program.programId);
const [userAccountPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("user"), wallet.publicKey.toBuffer()],
  program.programId
);

// Contest ID from slug
const contestId = Array.from(
  createHash("sha256").update("turf-totals-v1-matchday-1").digest()
);
```

### Settlement Test Pattern
```typescript
// Build remaining accounts for settlement
const remainingAccounts = settlements.map(s => [
  { pubkey: userAccountPda, isWritable: true, isSigner: false },
  { pubkey: entryPda, isWritable: true, isSigner: false },
]).flat();

await program.methods
  .settleContest(settlements)
  .accounts({ admin: admin.publicKey, vaultState: vaultPda, contest: contestPda })
  .remainingAccounts(remainingAccounts)
  .rpc();
```

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

### Current Deployment (Devnet)

- **Program ID**: `7Hy8GmJWPMdt6bx3VG4BLFnpNX9TBwkPt87W6bkHgr2J`
- **Vault PDA**: `7z313HTVNcxhvCBkkDQv794RpXeRrfCLb5WJ4dFAQQeh`
- **Admin**: `9Fy8P3DvKBh3awt1wr27g4CDh47oDqmJR2FAAQ1bc69D`
- **USDC Mint**: `222Dcu2RgAXE3T8A4mGSG3kQyXaNjqePx7vva1RdWBN9` (test, 6 decimals)
- **USDT Mint**: `9mxkN8KaVA8FFgDE2LEsn2UbYLPG8Xg9bf4V9MYYi8Ne` (test, 6 decimals)
- **IDL Account**: `DCP2XRu8ZwzsCpXBgu5xa4vTYdYQhKUZRU49iJuFv8Lf`

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

## Key Design Decisions

- **Custodial entry**: `enter_contest` separates payer (signer) from wallet (entry owner) — enables server-side entry on behalf of users
- **No lock instruction**: Contest can go directly from Open to Settled (Locked status exists but no instruction sets it yet)
- **Single admin**: One admin per vault, set at initialization. All contest ops require this admin
- **Dual mint**: USDC + USDT supported from day one, separate vault token accounts
- **Manual settlement**: No on-chain scoring — Rails computes results, admin submits final rankings

## Code Style

- Keep instructions in separate files under `instructions/`
- Thin wrappers in `lib.rs`, logic in instruction handlers
- All state in `state.rs`, all errors in `errors.rs`
- Use `msg!()` for on-chain logging at key events
- Prefer `require!()` macro over manual `if/return Err`
