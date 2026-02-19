# Issue #3788 Spec

- Title: Subtask: validate observability route parity matrix under secure and baseline modes
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Observability route behavior can drift between baseline and secure-mode serving paths unless matrix expectations and fail-closed policy markers are explicitly documented and contract-tested.

## Acceptance Criteria
- AC-1: Route parity matrix markers are documented for `/metrics`, `/healthz`, `/readyz`, and `/metrics.stream` across baseline and secure mode.
- AC-2: Policy drift markers for parity fail closed are documented and tested.
- AC-3: Unknown-path and malformed-method fail-closed route behaviors are covered by regression tests.
- AC-4: Unit, Functional, Integration, and Regression evidence is present and passing.

## Scope
In scope:
- `docs/observability/contracts.md`
- `crates/kamn-node/tests/observability_contracts_docs.rs`
- Existing route parity runtime tests in `crates/kamn-node/src/main_tests/observability_endpoint_tests.rs` as verification targets
- `specs/3788/{spec.md,plan.md,tasks.md}`

Out of scope:
- New endpoint schema design
- New shell policy scripts
- Dependency/protocol changes

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional | `docs/observability/contracts.md` route parity section | Baseline/secure route matrix and checkpoint markers are present |
| C-02 | AC-2 | Integration/Regression | `observability_contracts_docs` parity drift marker assertions | Policy drift markers fail closed on removal/mismatch |
| C-03 | AC-3 | Regression | unknown-path + malformed-method observability endpoint tests | Invalid routes/methods remain fail closed with deterministic 404 behavior |
| C-04 | AC-4 | Integration | baseline and secure-mode live route parity tests | Required routes pass in both baseline and TLS-required serving paths |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_serves_metrics_and_health_paths -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_returns_not_found_for_unknown_path -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_returns_not_found_for_malformed_request_method -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3788.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3788.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3788.json`

## Success Metrics
- Route parity and fail-closed drift markers are explicitly documented and contract-tested.
- Baseline + secure-mode route parity behavior remains green in integration/regression tests.
- No shell LOC increase for this issue (`shell_loc_delta_actual=0` target).
