# Token Model and Genesis Allocation Controls (Issues #136, #137)

This document captures the first implementation slice for PRD 9.1 token configuration.

## Scope Delivered
- Added `crates/kamn-core/src/token.rs` with:
  - `TokenConfig` model and validation.
  - `GenesisAllocation` model with deterministic bucket shares.
  - `AllocationBucket` enum for PRD allocation categories.
  - `default_token_config()` for KAMN symbol/supply/decimals/allocation defaults.
- Wired token config into bootstrap planning (`BootstrapPlan.token_config`) with validation.
- Added integration tests in `crates/kamn-core/tests/token_config.rs`.

## PRD Alignment
- Symbol: `KAMN`
- Total supply: `1,000,000,000`
- Decimals: `18`
- Allocation:
  - 40% ecosystem incentives
  - 25% protocol development
  - 20% validator rewards
  - 10% initial liquidity
  - 5% community grants

## Validation Rules
- Symbol must be uppercase alphanumeric.
- Total supply must be positive.
- Decimals must be <= 18.
- Allocation buckets must be unique.
- Allocation share sum must equal 10,000 bps.
- Allocation amount sum must equal total supply.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test token_config
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
