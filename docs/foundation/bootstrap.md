# Foundation Bootstrap (Issue #16)

This document describes the initial KAMN chain bootstrap scaffold.

## What Exists
- Rust workspace with two crates:
  - `kamn-core`: configuration, state namespaces, runtime wiring, bootstrap planner.
  - `kamn-node`: minimal binary entrypoint for processor/listener/approver bootstrap.
- Baseline state namespaces for key PRD domains:
  - DID registry
  - channels
  - messages
  - tasks
  - reputation
  - escrows
- App-state schema/version primitives:
  - `APP_STATE_VERSION` for the current schema revision.
  - `AppStateSchema` for version + namespace bundle.
  - `canonical_state_key(namespace, entity, id)` for deterministic key serialization and strict validation.
- Migration scaffolding:
  - `MigrationRegistry`, `MigrationStep`, and `MigrationPlan` for deterministic, contiguous state upgrades.
  - `bootstrap_from_state_version(...)` to compute the startup migration plan from persisted state to current schema.

## Local Validation
Run from repository root:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Run bootstrap binary example:

```bash
cargo run -p kamn-node -- --role processor --chain-id kamn-devnet --chain-version v0.1.0
```

## Notes
This is intentionally minimal and dependency-light so the bootstrap path is fast and auditable.
Migration execution is not implemented yet; this stage focuses on deterministic planning and validation hooks.
For role interaction baseline coverage, see `docs/foundation/role-smoke.md`.
For transaction invariant guard behavior, see `docs/foundation/transaction-guards.md`.
For canonical invariant IDs and taxonomy mapping, see `docs/foundation/invariants.md`.
For the Rust SDK first implementation slice, see `docs/foundation/rust-sdk-alpha.md`.
