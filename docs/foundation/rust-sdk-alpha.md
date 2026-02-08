# Rust SDK Alpha Slice (Issues #132, #133, #468)

This document describes the first implementation slice for the Rust SDK aligned to PRD 12.1.

## Scope Delivered
- Added new crate: `crates/kamn-sdk`.
- Added `KamnAgent` trait that covers first-slice identity, messaging, task, escrow, and discovery APIs.
- Added deterministic in-memory implementation: `InMemoryKamnClient`.
- Added deterministic `receive_stream(...)` iterator API for async-style receive workflows.
- Added typed SDK data models and explicit error types.
- Added integration-style tests for register/resolve, send/receive, task lifecycle, escrow lifecycle, search, and reputation.

## Implementation Notes
- No new third-party dependencies were introduced.
- The implementation is dependency-light and optimized for fast local/CI feedback.
- The in-memory adapter is intentionally deterministic and suitable for early SDK contract validation.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-sdk
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Follow-up
- Introduce async transport-backed client implementation in a later slice.
- Extend stream API parity to Python and TypeScript SDK implementations.
