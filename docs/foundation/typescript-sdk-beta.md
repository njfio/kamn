# TypeScript SDK Beta and Shared Schema Package (Issues #218, #219, #485, #585, #634)

This document captures the first TypeScript SDK implementation slice and the shared protocol schema package used to keep language SDK behavior aligned.

## Scope Delivered
- Added `packages/kamn-schema` with canonical message envelope primitives:
  - constants for canonical type, encryption algorithm, and proof purpose.
  - `createCanonicalMessageEnvelope(...)` helper.
  - `validateCanonicalMessageEnvelope(...)` strict validation rules.
  - `canonicalPayload(...)` deterministic payload serialization.
- Added `packages/kamn-sdk` with dependency-light in-memory SDK parity:
  - `KAMNClient` for register/resolve/send/receive/receiveStream/task/escrow/search/reputation flows.
  - `SDKError` explicit typed errors.
  - `LiveTransportConfig` and `LiveTransportKAMNClient` for endpoint-scoped live transport parity.
  - `TransportModeMismatchError` for typed transport mode mismatch rejection (`Regression: #620`).
  - send path enforces schema validation through `kamn-schema`.

## Shared Schema Rules (Parity Targets)
The TypeScript schema package mirrors canonical constraints used by core protocol docs:
- envelope type must be `kamn:message:v1`.
- sender and recipients must be valid `kamn:did:agent:*` values.
- expiry must be strictly after creation.
- nonce must be a positive integer.
- message type must be in canonical allowed set.
- encryption algorithm must be `X25519-XChaCha20-Poly1305`.
- recipient keys and body entries must be non-empty.
- proof purpose must be `authentication`.
- proof verification method must be bound to sender DID (`<from>#...`).

## TypeScript SDK Beta Behavior
- IDs are deterministic (`agent_<n>`, `msg_<n>`, `task_<n>`, `escrow_<n>`).
- inbox reads are draining by design.
- async `receiveStream(...)` yields drained inbox messages in deterministic order.
- escrow release is one-way and idempotency-protected.
- search results are deterministic and sorted by DID.
- schema violations in send path are surfaced as `SDKError`.
- transport mode mismatch (`in-memory` vs `live`) is surfaced through `TransportModeMismatchError` (`Regression: #620`).

## Fast and Cost-Effective Validation
This slice avoids dependency-heavy toolchains and uses Node 22 native TypeScript stripping:
- `node --experimental-strip-types --test ...`

PR-fast validation commands:

```bash
bash scripts/sdk/run_live_transport_parity_contract_lane.sh
npm --prefix packages/kamn-schema test
npm --prefix packages/kamn-sdk test
bash scripts/sdk/test_run_sdk_parity_matrix.sh
```

These tests are deterministic, run in milliseconds, and require no package install at this stage.

## Local Validation
Run from repository root:

```bash
bash scripts/sdk/run_live_transport_parity_contract_lane.sh
bash scripts/sdk/run_live_transport_parity_deep_lane.sh
npm --prefix packages/kamn-schema test
npm --prefix packages/kamn-sdk test
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test docs_contract_matrix_wave2_harness
cargo test -p kamn-core
```

## Shared SDK Parity Fixture Matrix
The TypeScript SDK participates in shared cross-language parity checks:

- Fixture source: `fixtures/sdk_parity/register_validation_cases.json`
- Matrix command:
  - `bash scripts/sdk/run_sdk_parity_matrix.sh --fixture fixtures/sdk_parity/register_validation_cases.json --output-json /tmp/sdk-parity-report.json`

## SDK Example Fixture Drift Checker Contract
Deterministic snapshot drift checks keep generated fixture artifacts aligned with runtime behavior:

- Snapshot source: `fixtures/sdk_parity/register_validation_snapshot.json`
- Drift checker command:
  - `python3 scripts/sdk/check_example_fixture_drift.py --fixture fixtures/sdk_parity/register_validation_cases.json --snapshot fixtures/sdk_parity/register_validation_snapshot.json --output-json /tmp/sdk-example-fixture-drift-report.json`
- Policy checker command:
  - `bash scripts/sdk/check_example_fixture_drift_policy.sh --report-file /tmp/sdk-example-fixture-drift-report.json`
- Contract lane command:
  - `bash scripts/sdk/run_example_fixture_drift_contract_lane.sh --output-report /tmp/sdk-example-fixture-drift-contract-report.json`

Fixture snapshot mismatch or policy schema drift is fail-closed (`Regression: #940`).
