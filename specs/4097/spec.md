# Issue #4097 Spec

- Title: Subtask: add docs-runbook and go-no-go marker parity contracts for overload governance
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Overload governance evidence can drift when `docs/ci/strategy.md`, `docs/ops/configuration.md`, and stress-runner reason markers are not contract-bound to the same go/no-go taxonomy.

## Acceptance Criteria
- AC-1: Docs/runbook parity markers for overload governance are explicitly documented in `docs/ci/strategy.md`.
- AC-2: Go/no-go reason marker taxonomy for overload stress governance remains deterministic and fail-closed.
- AC-3: Remediation marker entries exist for each overload go/no-go reason code.
- AC-4: Unit/Functional/Integration/Regression tests for overload docs parity and marker drift pass.

## Scope
In scope:
- Add overload docs parity and go/no-go marker contract section in `docs/ci/strategy.md`.
- Add overload remediation marker references to `docs/ops/configuration.md`.
- Add Rust docs-contract tests that enforce:
  - strategy marker presence,
  - strategy/ops/runner marker parity,
  - remediation-per-reason coverage.
- Add issue lifecycle artifacts in `specs/4097/{spec.md,plan.md,tasks.md}`.

Out of scope:
- New shell/python CI checker scripts.
- Runtime scheduler or stress-runner behavioral changes.
- Workflow topology changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 120
- shell_to_rust_ratio_delta_estimate: -0.0006
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `docs/ci/strategy.md` overload parity section | required overload docs parity markers are present and deterministic |
| C-02 | AC-2 | Integration | strategy markers + `docs/ops/configuration.md` + `scripts/ci/run_daemon_os_signal_stress_matrix.sh` | taxonomy/schema/reason markers remain synchronized |
| C-03 | AC-3 | Regression | reason-code CSV from strategy marker block | each reason code has a remediation marker entry |
| C-04 | AC-4 | Conformance | targeted docs-contract test commands | overload docs parity tests pass with fail-closed drift checks |

## Test Mapping
- C-01 -> `cargo test -p kamn-core --test ci_strategy_docs doc_contains_overload_docs_parity_and_go_no_go_markers -- --exact`
- C-02 -> `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_matches_ops_docs_and_runner_markers -- --exact`
- C-03 -> `cargo test -p kamn-core --test ci_strategy_docs doc_enforces_overload_docs_parity_requires_remediation_marker_for_each_reason_code -- --exact`
- C-04 -> `cargo test -p kamn-core --test ci_strategy_docs --test service_api_ops_configuration_docs`

## Success Metrics
- Overload go/no-go docs parity becomes deterministic and test-enforced.
- Marker drift across strategy/ops/runner surfaces fails closed in Rust docs-contract tests.
- Shell LOC remains unchanged for this subtask.
