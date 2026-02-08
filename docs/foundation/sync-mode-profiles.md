# Sync-Mode Operational Profiles (Issues #208, #209)

This document captures the first implementation slice for startup sync profiles and recovery behavior selection.

## Scope Delivered
- Added sync profile primitives in `crates/kamn-core/src/config.rs`:
  - `SyncMode`: `fast`, `slow`, `archive`
  - `SyncStartupStrategy` and `SyncRecoveryStrategy`
  - `SyncOperationalProfile` mapping from `SyncMode::profile()`
- Extended `NodeConfig` with `sync_mode` and `operational_profile()`.
- Added CLI wiring in `crates/kamn-node/src/main.rs`:
  - new flag: `--sync-mode <fast|slow|archive>`
  - default: `fast`
- Added integration tests in `crates/kamn-core/tests/sync_mode_profiles.rs`.

## Profile Semantics
- `fast`:
  - Startup: state-sync to latest known state.
  - Recovery: resume from recent state and continue.
  - Version guard: relaxed.
  - History: does not require full chain history.
- `slow`:
  - Startup: block replay from genesis.
  - Recovery: replay missing blocks.
  - Version guard: strict chain/code version alignment required.
  - History: no archive guarantee.
- `archive`:
  - Startup: archive-oriented state sync from genesis horizon.
  - Recovery: replay archived history.
  - Version guard: relaxed.
  - History: full history retention expected.

## Kolme Alignment Reference
The profile split follows Kolme sync-manager concepts:
- block sync mode
- state sync mode
- archive mode

KAMN names this first slice as `slow`, `fast`, and `archive` to keep operator-facing semantics direct.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test sync_mode_profiles
cargo test -p kamn-node
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
