# Issue #3781 Spec

- Title: Task: enforce secure observability route parity and fail-closed drift policy
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Observability route behavior and secure-serving requirements needed explicit fail-closed contracts and deterministic drift markers to prevent production regression during runtime hardening.

## Acceptance Criteria
- AC-1: Secure-serving policy for observability routes fails closed on invalid/missing contract markers.
- AC-2: Route parity checks pass for `/metrics`, `/healthz`, `/readyz`, and `/metrics.stream` across baseline and secure mode.
- AC-3: Drift checker captures marker/taxonomy regressions with deterministic reason codes.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Scope
In scope:
- Child subtask delivery and closure:
  - `#3789` secure-mode config fail-closed validation
  - `#3788` route parity matrix validation under baseline + secure mode
- Contract documentation:
  - `docs/foundation/runtime-network.md`
  - `docs/observability/contracts.md`
- Docs-contract and runtime verification in `kamn-node` observability test suites
- `specs/3781/{spec.md,plan.md,tasks.md}`

Out of scope:
- External certificate automation and ingress-controller provisioning
- New observability schema design
- Additional shell lane surface expansion

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | TLS secure-mode negative matrix tests (invalid mode/missing cert/invalid key/plain HTTP rejection) | Invalid secure-mode states fail closed with deterministic markers |
| C-02 | AC-2 | Integration | Baseline and TLS-required route parity integration tests | Required observability routes serve expected status/content-type across both modes |
| C-03 | AC-3 | Unit/Regression | Docs-contract assertions for parity/taxonomy drift markers | Marker/taxonomy drift fails closed |
| C-04 | AC-4 | Regression | lint + guardrails | Quality gates pass with no shell-surface regression |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_serves_metrics_and_health_paths -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_returns_not_found_for_unknown_path -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_returns_not_found_for_malformed_request_method -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_missing_cert_file -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_key_file -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_mode_value -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3781.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3781.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3781.json`

## Success Metrics
- Parent task ACs fully satisfied by merged child subtasks `#3788` and `#3789`.
- Deterministic secure-serving and parity contracts remain pinned in docs and tests.
- No shell LOC increase for this closure issue (`shell_loc_delta_actual=0` target).
