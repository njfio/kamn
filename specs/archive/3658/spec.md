# Issue #3658 Spec

- Title: `Subtask: add observability-over-TLS integration checks and local-heavy lane`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Observability route contracts currently focus on plaintext serving and do not provide explicit integration evidence for `/metrics`, `/healthz`, and `/readyz` over HTTPS.

## Scope
In:
- Add observability endpoint TLS integration checks for `/metrics`, `/healthz`, and `/readyz`.
- Extend local-heavy observability validation lane to include deterministic TLS route marker coverage.
- Add policy/contract-lane checks that fail closed on TLS route marker drift.
- Update CI strategy docs for observability TLS route lane behavior.

Out:
- Certificate issuance automation.
- TLS negative-path matrix breadth (invalid cert/key/mode mismatch) handled by `#3805`.

## Acceptance Criteria
- AC-1: Given observability endpoint TLS mode, when HTTPS clients call `/metrics`, `/healthz`, and `/readyz`, then each route returns contract-compatible status/content semantics.
- AC-2: Given local-heavy observability lane execution, when lane reports are produced, then deterministic TLS route markers are emitted and validated in policy/contract checks.
- AC-3: Given tampered lane reports with TLS marker drift, when policy is evaluated, then policy fails closed with deterministic reason markers.
- AC-4: Given observability TLS lane updates, when docs are checked, then CI strategy commands/markers reflect the TLS route coverage.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes -- --exact --nocapture` | HTTPS route contract coverage for `/metrics`, `/healthz`, `/readyz` passes |
| C-02 | AC-2 | Functional/Conformance | `bash scripts/runtime/test_validate_runtime_observability_endpoint_live.sh` | Runtime observability lane emits and validates TLS marker |
| C-03 | AC-2/AC-3 | Functional/Regression | `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh` | TLS marker drift is rejected fail-closed |
| C-04 | AC-2 | Integration/Conformance | `bash scripts/runtime/test_validate_local_observability_scrape_live.sh` | Local-heavy lane emits TLS marker and includes TLS selector coverage |
| C-05 | AC-4 | Docs/Conformance | `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh` | CI strategy marker/docs parity remains verified |

## Test Mapping
- `crates/kamn-node/src/observability_endpoint.rs`
- `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs`
- `scripts/runtime/validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/runtime_observability_endpoint_live_contract.py`
- `scripts/runtime/local_observability_scrape_live_contract.py`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `scripts/runtime/test_validate_local_observability_scrape_live.sh`
- `scripts/runtime/test_check_local_observability_scrape_live_policy.sh`
- `scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh`

## Success Metrics
- Observability TLS route integration checks are deterministic and green.
- Local-heavy lane reports/policy enforce TLS route marker parity fail-closed.
- CI strategy docs explicitly cover observability TLS lane markers and command surface.
