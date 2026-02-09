# Python SDK Beta Slice (Issues #134, #135, #483, #585, #634)

This document captures the first Python SDK implementation slice for MVP workflow parity.

## Scope Delivered
- Added dependency-light Python SDK module: `kamn_sdk.py`.
- Added in-memory `KAMNClient` implementation with:
  - `register`, `resolve`
  - `send`, `receive`, `receive_stream` (async generator)
  - `create_task`, `accept_task`
  - `create_escrow`, `release_escrow`, `balance`
  - `search_agents`, `get_reputation`
- Added live transport parity implementation:
  - `LiveTransportConfig` endpoint contract.
  - `LiveKAMNClient` endpoint-scoped shared-state live path.
  - transport mode contracts (`TransportMode`) and mismatch rejection (`TransportModeMismatchError`, `Regression: #620`).
- Added Python test coverage: `tests/python/test_sdk.py`.
- Added scheduled deep-lane coverage: `tests/python/test_sdk_live_transport_deep.py`.

## Deterministic Behavior
- DIDs, message IDs, task IDs, and escrow IDs use deterministic sequence generators.
- Inbox receives are drain-based and idempotent on repeated reads.
- Async `receive_stream(...)` yields drained messages in deterministic FIFO order.
- Escrow release is one-time and guarded against duplicate release.
- Unknown DID/task/escrow access returns explicit `SDKError`.
- Transport mode mismatch (`in-memory` vs `live`) is explicitly rejected to prevent cross-language contract drift (`Regression: #620`).

## Local Validation
Run from repository root:

```bash
bash scripts/sdk/run_live_transport_parity_contract_lane.sh
bash scripts/sdk/run_live_transport_parity_deep_lane.sh
python3 -m unittest tests/python/test_sdk.py
bash scripts/sdk/test_run_sdk_parity_matrix.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Shared SDK Parity Fixture Matrix
The Python SDK participates in shared cross-language parity checks:

- Fixture source: `fixtures/sdk_parity/register_validation_cases.json`
- Matrix command:
  - `bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json /tmp/sdk-parity-report.json`

## SDK Example Fixture Drift Checker Contract
Snapshot drift is guarded by deterministic checker + policy commands:

- Snapshot source: `fixtures/sdk_parity/register_validation_snapshot.json`
- Drift checker command:
  - `python3 scripts/sdk/check_example_fixture_drift.py --fixture fixtures/sdk_parity/register_validation_cases.json --snapshot fixtures/sdk_parity/register_validation_snapshot.json --output-json /tmp/sdk-example-fixture-drift-report.json`
- Policy checker command:
  - `bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file /tmp/sdk-example-fixture-drift-report.json`
- Contract lane command:
  - `bash scripts/sdk/run_example_fixture_drift_contract_lane.sh --output-report /tmp/sdk-example-fixture-drift-contract-report.json`

Snapshot/report drift is fail-closed (`Regression: #940`).
