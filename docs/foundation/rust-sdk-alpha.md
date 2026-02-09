# Rust SDK Alpha Slice (Issues #132, #133, #468, #585, #633)

This document describes the first implementation slice for the Rust SDK aligned to PRD 12.1.

## Scope Delivered
- Added new crate: `crates/kamn-sdk`.
- Added `KamnAgent` trait that covers first-slice identity, messaging, task, escrow, and discovery APIs.
- Added deterministic in-memory implementation: `InMemoryKamnClient`.
- Added deterministic `receive_stream(...)` iterator API for async-style receive workflows.
- Added live transport adapter implementation: `LiveTransportKamnClient`.
- Added `LiveTransportConfig` endpoint contract for live transport wiring.
- Added explicit transport mode contracts (`TransportMode`, `KamnTransport`) with mismatch rejection (`Regression: #620`).
- Added typed SDK data models and explicit error types.
- Added integration-style tests for register/resolve, send/receive, task lifecycle, escrow lifecycle, search, and reputation.
- Added live transport tests for functional, integration, regression, and performance lanes.

## Implementation Notes
- No new third-party dependencies were introduced.
- The implementation is dependency-light and optimized for fast local/CI feedback.
- The in-memory adapter is intentionally deterministic and suitable for early SDK contract validation.
- Live transport adapter uses endpoint-scoped shared state to provide deterministic contract coverage without introducing new runtime dependencies.

## Local Validation
Run from repository root:

```bash
bash scripts/sdk/run_local_e2e_demo.sh
bash scripts/sdk/run_localhost_signed_demo.sh
bash scripts/sdk/run_tcp_signed_relay_demo.sh
bash scripts/sdk/run_rust_live_transport_contract_lane.sh
bash scripts/sdk/run_rust_live_transport_deep_lane.sh
cargo test -p kamn-sdk
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Localhost Signed Two-Process Demo (Issue #807)
The SDK now includes a localhost two-process signed-message demo:

- One-command orchestrator:
  - `bash scripts/sdk/run_localhost_signed_demo.sh`
- Manual terminal split:
  - Listener terminal:
    - `cargo run --quiet -p kamn-sdk --example localhost_signed_listener -- --addr 127.0.0.1:17879`
  - Sender terminal:
    - `cargo run --quiet -p kamn-sdk --example localhost_signed_sender -- --addr 127.0.0.1:17879 --body "hello-from-localhost-demo"`
- Expected markers:
  - `status=ok` (sender and listener)
  - `verified=true`
  - `signature=sig:ed25519:baseline-v1:...`

`Regression: #807` ensures the command remains executable and preserves signed-message verification markers.

## TCP Signed Relay Adapter Demo (Issue #822)
The SDK now includes a deterministic TCP relay adapter demo backed by reusable wire framing/parsing helpers:

- One-command orchestrator:
  - `bash scripts/sdk/run_tcp_signed_relay_demo.sh`
- Manual terminal split:
  - Listener terminal:
    - `cargo run --quiet -p kamn-sdk --example tcp_signed_relay_listener -- --addr 127.0.0.1:17881`
  - Sender terminal:
    - `cargo run --quiet -p kamn-sdk --example tcp_signed_relay_sender -- --addr 127.0.0.1:17881 --body "hello-from-tcp-relay-demo"`
- Expected markers:
  - `status=ok` (sender and listener)
  - `verified=true`
  - `adapter=tcp`
  - `signature=sig:ed25519:baseline-v1:...`

`Regression: #822` ensures deterministic envelope parsing rejects malformed payloads and preserves signed relay markers.

## Local End-to-End Demo (Issue #770)
The SDK now includes a deterministic one-command demo for first-run validation:

- Command: `bash scripts/sdk/run_local_e2e_demo.sh`
- Expected markers:
  - `status=ok`
  - `message_id=<id>`
  - `task_id=<id>`
  - `artifact_id=<id>`
  - `escrow_id=<id>`

`Regression: #770` ensures the command remains executable and emits lifecycle markers.

## Follow-up
- Replace endpoint-scoped deterministic live adapter with real network transport client implementation in a later slice.
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

## CI Routing Policy (Issue #635)
PR checks use changed-language routing to keep cost and runtime bounded:

- Rust SDK diffs route to `bash scripts/sdk/run_rust_live_transport_contract_lane.sh`.
- Python SDK diffs route to `python3 -m unittest tests/python/test_sdk.py`.
- TypeScript SDK diffs route to `npm --prefix packages/kamn-sdk test`.
- Multi-language SDK diffs (or parity-lane runner changes) route to
  `bash scripts/sdk/run_live_transport_parity_contract_lane.sh`.
- Shared fixture and matrix runner diffs route to
  `bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json /tmp/sdk-parity-report.json`.

Deep-lane entrypoints are defined for scheduled execution and are intentionally excluded from the fast PR lane:

- `bash scripts/sdk/run_rust_live_transport_deep_lane.sh`
- `bash scripts/sdk/run_live_transport_parity_deep_lane.sh`
