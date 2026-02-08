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
