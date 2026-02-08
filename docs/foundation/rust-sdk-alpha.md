# Rust SDK Alpha Slice (Issues #132, #133, #468, #585)

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

## Shared SDK Parity Fixture Matrix
Cross-language behavior parity is validated with a shared fixture corpus:

- Fixture source: `fixtures/sdk_parity/register_validation_cases.json`
- Matrix command:
  - `bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json /tmp/sdk-parity-report.json`

Runner entrypoints:
- Rust: `bash scripts/sdk/run_parity_rust.sh`
- Python: `bash scripts/sdk/run_parity_python.sh`
- TypeScript: `bash scripts/sdk/run_parity_typescript.sh`
