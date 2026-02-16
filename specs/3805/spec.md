# Issue #3805 Spec

- Title: `Subtask: add observability TLS negative-matrix fail-closed coverage`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Observability-over-TLS currently verifies positive route behavior, but it lacks deterministic negative-matrix coverage for invalid certificate/key material, TLS mode mismatch, and rejected non-TLS handshakes.

## Scope
In:
- Add deterministic negative-path tests for observability TLS mode with invalid cert/key and invalid mode mismatch.
- Add fail-closed marker outputs for TLS negative matrix in runtime observability live validation and policy/contract-lane checks.
- Add deterministic policy tamper checks for TLS negative marker drift.
- Document observability TLS negative-path taxonomy in runtime network docs.

Out:
- Certificate issuance automation and rotation workflows.
- mTLS handshake matrix expansion.

## Acceptance Criteria
- AC-1: Given TLS mode `require` and missing/invalid cert material, when observability endpoint startup is evaluated, then it fails closed with deterministic reason markers.
- AC-2: Given TLS mode `require` and invalid key material, when startup is evaluated, then it fails closed with deterministic reason markers.
- AC-3: Given invalid TLS mode values or mode mismatch input, when startup or policy checks run, then deterministic fail-closed markers are emitted.
- AC-4: Given plain HTTP traffic sent to TLS observability endpoint, when handshake is rejected, then route contract remains fail-closed and validated by tests.
- AC-5: Given runtime-network docs, when checked, then observability TLS negative taxonomy and marker contracts are documented.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Integration | `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_missing_cert_file -- --exact --nocapture` | Missing cert fails closed with deterministic error marker |
| C-02 | AC-2 | Unit/Integration | `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_key_file -- --exact --nocapture` | Invalid key fails closed with deterministic error marker |
| C-03 | AC-3 | Unit/Functional | `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_mode_value -- --exact --nocapture` | Invalid TLS mode value fails closed deterministically |
| C-04 | AC-4 | Integration | `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_rejects_plain_http_handshake -- --exact --nocapture` | Plain HTTP handshake to TLS endpoint is rejected fail-closed |
| C-05 | AC-1/AC-2/AC-3 | Functional/Conformance | `bash scripts/runtime/test_validate_runtime_observability_endpoint_live.sh` | Runtime validation emits deterministic TLS negative matrix marker |
| C-06 | AC-3 | Functional/Regression | `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh` | TLS negative matrix marker drift fails closed deterministically |
| C-07 | AC-3/AC-5 | Integration/Conformance | `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh` | Contract lane report includes TLS negative marker and docs parity |

## Test Mapping
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- `scripts/runtime/validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/runtime_observability_endpoint_live_contract.py`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/validate_runtime_observability_endpoint_live_contract_lane.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `docs/foundation/runtime-network.md`

## Success Metrics
- Negative TLS matrix failures are deterministic and fail closed.
- Runtime observability policy and contract lane reject TLS negative marker drift.
- Runtime-network docs include observability TLS negative-path taxonomy and marker contracts.
