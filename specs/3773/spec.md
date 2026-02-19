# Issue #3773 Spec

- Title: Story: standardize tracing and harden observability serving contracts
- Status: Implemented
- Type: story
- Priority: P0
- Milestone: specs/milestones/r26-5-observability-and-transport-resilience-hardening/index.md

## Problem Statement
Production operations needed deterministic runtime tracing plus hardened observability-serving contracts so incident diagnosis remains reliable and fail closed as the runtime evolves.

## Acceptance Criteria
- AC-1: Runtime/service logging is standardized with deterministic structured fields and level controls.
- AC-2: Observability endpoints enforce secure-serving contracts and parity across `/metrics`, `/healthz`, `/readyz`, `/metrics.stream`.
- AC-3: Drift/policy checks fail closed when observability route markers or security-mode contracts regress.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Scope
In scope:
- Child task deliveries:
  - `#3775` tracing bootstrap + runtime event taxonomy
  - `#3781` secure observability route parity + fail-closed drift policy
  - `#3776` local-heavy observability lane governance contracts
- Story-level traceability artifacts:
  - `specs/3773/{spec.md,plan.md,tasks.md}`
- Verification surfaces:
  - `docs/observability/contracts.md`
  - `docs/foundation/runtime-network.md`
  - `docs/ci/strategy.md`
  - `kamn-node` observability/runtime tests and `kamn-core` CI strategy docs-contract tests

Out of scope:
- Centralized SaaS observability backend provisioning
- Non-runtime analytics redesign

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | runtime tracing bootstrap/full tests + docs-contract taxonomy checks | deterministic tracing bootstrap and taxonomy markers remain stable |
| C-02 | AC-2 | Integration | baseline + secure-mode observability route tests | route parity and secure serving pass across required surfaces |
| C-03 | AC-3 | Regression | TLS negative-matrix + parity drift docs-contract markers + CI strategy docs contract checks | regressions fail closed with deterministic markers |
| C-04 | AC-4 | Regression | lint + shell guardrails | quality/governance gates remain green |

## Test Mapping
- `cargo test -p kamn-node --test observability_contracts_docs`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_bootstrap_runtime_emits_structured_marker -- --exact`
- `cargo test -p kamn-node main_tests::runtime_tests::integration_runtime_full_emits_ordered_bootstrap_readiness_markers -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_serves_metrics_and_health_paths -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes -- --exact`
- `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_mode_value -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-node --all-targets -- -D warnings`
- `scripts/ci/check_shell_loc_hard_ceiling.sh --output-json /tmp/shell-loc-hard-ceiling-3773.json`
- `scripts/ci/check_shell_rust_ratio_guardrail.sh --output-json /tmp/shell-rust-ratio-3773.json`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh --output-json /tmp/shell-threshold-ratchet-3773.json`

## Success Metrics
- Story ACs fully satisfied by merged child tasks `#3775`, `#3781`, and `#3776`.
- Runtime tracing and observability serving contracts remain deterministic and fail closed.
- No shell LOC increase for this closure issue (`shell_loc_delta_actual=0` target).
