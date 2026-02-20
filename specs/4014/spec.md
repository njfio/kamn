# Issue #4014 Spec

- Title: Task: enforce ci durability governance checker with baseline drift and docs parity contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement
Durability governance now has deterministic sqlite crash-recovery dry-run artifacts (`validate`, `policy`, and contract lane/evidence convergence), but CI lacks a single fail-closed dry-run governance checker that validates baseline thresholds, selector/workflow exclusion policy, and docs/runbook parity/remediation markers. Without this gate, drift can pass through fast-gate before release promotion.

## Scope
In scope:
- Add a CI dry-run durability governance checker under `scripts/ci/` that validates:
  - sqlite crash-recovery dry-run report contract parity,
  - threshold/baseline fixture integrity and runtime budget bounds,
  - ci-tools fast-mode selector and ci-fast-gate workflow exclusion drift,
  - docs and runbook marker parity plus remediation marker coverage.
- Add deterministic baseline threshold fixture under `fixtures/ci/`.
- Add Rust contract tests with unit/functional/integration/regression/performance coverage.
- Update `docs/ci/strategy.md` and `docs/ops/configuration.md` with checker markers and remediation mappings.
- Wire checker contract test into `scripts/ci/test_ci_tools.sh` fast/full paths.

Out of scope:
- Adding new heavy sqlite crash-recovery run-mode execution in CI.
- Changing sqlite crash-recovery runtime lane semantics.
- Workflow topology redesign.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 8
- rust_loc_delta_estimate: 430
- shell_to_rust_ratio_delta_estimate: -0.0008
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: CI checker fails closed when sqlite crash-recovery dry-run report contracts drift (schema/status/final-decision/taxonomy/reason markers).
- AC-2: CI checker fails closed when threshold fixture contracts drift or runtime budget thresholds are exceeded.
- AC-3: Selector policy keeps sqlite crash-recovery run-mode commands out of ci-tools fast-mode and ci-fast-gate workflow.
- AC-4: Docs/runbook marker parity remains deterministic, including remediation markers for every checker reason code.
- AC-5: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | checker + valid sqlite dry-run reports | checker returns `status=pass`, `final_decision=GO`, deterministic contract markers |
| C-02 | AC-1 | Functional | checker + tampered sqlite report marker | checker returns `NO-GO` with deterministic report-contract drift reason |
| C-03 | AC-2 | Unit | checker + durability threshold fixture | required keys parse deterministically; missing/invalid keys fail closed |
| C-04 | AC-3 | Integration | checker + `scripts/ci/test_ci_tools.sh` + `.github/workflows/ci-fast-gate.yml` | required fast-mode entry present; run-mode leakage rejected |
| C-05 | AC-4 | Regression | checker + docs/remediation marker drift fixture | checker fails closed on docs marker parity/remediation drift |
| C-06 | AC-5 | Performance | checker + valid baseline inputs | checker runtime stays within threshold and reports deterministic budget markers |

## Test Mapping
- `cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract unit_sqlite_crash_recovery_ci_dry_run_checker_accepts_valid_reports -- --exact`
- `cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract functional_sqlite_crash_recovery_ci_dry_run_checker_rejects_tampered_report_contract -- --exact`
- `cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract integration_sqlite_crash_recovery_ci_dry_run_checker_enforces_selector_and_workflow_exclusion -- --exact`
- `cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract regression_sqlite_crash_recovery_ci_dry_run_checker_rejects_docs_remediation_parity_drift -- --exact`
- `cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract performance_sqlite_crash_recovery_ci_dry_run_checker_stays_within_budget -- --exact`

## Success Metrics
- Durability dry-run governance contracts fail closed deterministically under report/threshold/selector/docs drift.
- sqlite crash-recovery run-mode exclusion remains explicit and test-backed in fast-gate surfaces.
- Strategy + ops docs remain synchronized with checker taxonomy, thresholds, and remediation markers.
