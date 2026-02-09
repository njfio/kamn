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
bash scripts/sdk/run_tcp_failover_reconnect_matrix.sh --lane fast --output-json /tmp/kamn-tcp-failover-reconnect-report.json
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
  - handshake profile validation uses `ed25519:baseline-v1`

Security guard behavior now fails closed across reconnect cycles:

- Forged handshake frames are rejected with explicit handshake field classification.
- Replayed nonces for the same `(from, to)` route are rejected with:
  - `conflict: tcp handshake replay detected`

`Regression: #822` ensures deterministic envelope parsing rejects malformed payloads and preserves signed relay markers.
`Regression: #823` ensures forged handshake frames and replayed nonces are rejected across reconnect.

## TCP Failover/Reconnect Integration Matrix (Issue #824)
Deterministic failover matrix coverage is available as a bounded fast lane with optional scheduled deep expansion:

- Fast lane command:
  - `bash scripts/sdk/run_tcp_failover_reconnect_matrix.sh --lane fast --output-json /tmp/kamn-tcp-failover-reconnect-report.json`
- Deep lane (scheduled-only path through `run_rust_live_transport_deep_lane.sh`):
  - `KAMN_TCP_FAILOVER_DEEP_CADENCE=scheduled bash scripts/sdk/run_rust_live_transport_deep_lane.sh`

Fast-lane report schema marker:

- `schema_version=kamn.sdk.tcp-failover-reconnect.matrix.v1`

Regression fixture path:

- `fixtures/sdk_failover_reconnect/reconnect_drift_signatures.txt`

`Regression: #824` ensures reconnect drift signatures remain fail-closed and matrix reporting stays deterministic.

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

## SDK Schema Compatibility Contract
Schema compatibility evidence is generated from the shared parity fixture and checked with a deterministic policy contract:

- Fixture source: `fixtures/sdk_parity/register_validation_cases.json`
- Matrix command:
  - `bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json /tmp/sdk-parity-report.json`
- Contract lane command:
  - `bash scripts/sdk/run_sdk_schema_compatibility_contract_lane.sh --output-file /tmp/sdk-schema-compatibility-contract-bundle.json`
- Policy checker command:
  - `bash scripts/sdk/check_sdk_schema_compatibility_policy.sh --bundle-file /tmp/sdk-schema-compatibility-contract-bundle.json`

Required schema markers:

- `kamn.sdk.parity.matrix.v1`
- `kamn.sdk.schema-compatibility-evidence.v1`

The contract fails closed: schema-version drift, case mismatch, or tampered reason codes force `NO-GO` (`Regression: #937`).

## SDK Example Fixture Drift Checker Contract
Generated SDK example fixture outcomes are locked to a deterministic snapshot:

- Snapshot source:
  - `fixtures/sdk_parity/register_validation_snapshot.json`
- Drift checker command:
  - `python3 scripts/sdk/check_example_fixture_drift.py --fixture fixtures/sdk_parity/register_validation_cases.json --snapshot fixtures/sdk_parity/register_validation_snapshot.json --output-json /tmp/sdk-example-fixture-drift-report.json`
- Policy checker command:
  - `bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file /tmp/sdk-example-fixture-drift-report.json`
- Contract lane command:
  - `bash scripts/sdk/run_example_fixture_drift_contract_lane.sh --output-report /tmp/sdk-example-fixture-drift-contract-report.json`

The lane fails closed on snapshot drift or report-schema mismatch (`Regression: #940`).

## Live Transport Smoke Parity Budget Contract
A bounded smoke-parity lane is available for deterministic cross-SDK live transport checks with explicit runtime/retry guardrails:

- Smoke lane command:
  - `bash scripts/sdk/run_live_transport_smoke_parity_lane.sh --output-json /tmp/sdk-live-transport-smoke-report.json`
- Policy checker command:
  - `bash scripts/sdk/check_live_transport_smoke_parity_policy.sh --report-file /tmp/sdk-live-transport-smoke-report.json`
- Contract lane command:
  - `bash scripts/sdk/run_live_transport_smoke_parity_contract_lane.sh --output-file /tmp/sdk-live-transport-smoke-contract-report.json`

Runtime/retry budget controls:

- `KAMN_SDK_SMOKE_PARITY_MAX_SECONDS`
- `KAMN_SDK_SMOKE_PARITY_MAX_RETRIES`

Smoke report schema marker:

- `kamn.sdk.live-transport-smoke-parity-report.v1`

The lane fails closed: retry-budget exhaustion, runtime-budget breaches, or transport parity drift force `NO-GO` (`Regression: #938`).

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
