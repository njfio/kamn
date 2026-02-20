# Issue #4044 Spec

- Title: Task: add ci dry-run compatibility governance checker and docs-runbook parity contracts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Problem Statement
Compatibility governance now has three deterministic lanes (`#4041`, `#4042`, `#4043`) but no single CI dry-run checker that fails closed on cross-lane drift, threshold drift, and docs/runbook marker parity drift. Without that gate, compatibility regressions can pass through fast-gate composition unnoticed.

## Scope
In scope:
- Add a CI dry-run compatibility governance checker under `scripts/ci/` that validates:
  - lane schema/final-decision/taxonomy contracts,
  - runtime budget thresholds from a fixture baseline,
  - fast-gate selector leakage (heavy run-mode exclusion),
  - docs/runbook marker parity and deterministic remediation markers.
- Add baseline threshold fixture under `fixtures/ci/`.
- Add Rust contract tests with unit/functional/integration/regression/performance coverage.
- Update `docs/ci/strategy.md` and `docs/ops/configuration.md` with checker/runbook parity markers.
- Wire checker contract test into `scripts/ci/test_ci_tools.sh` fast/full paths.

Out of scope:
- New heavy compatibility execution lanes in CI.
- Runtime protocol migration features.
- Workflow topology redesign.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 6
- rust_loc_delta_estimate: 420
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: CI checker fails closed when compatibility lane contracts drift (schema/status/final decision/reason taxonomy/reason codes).
- AC-2: CI checker fails closed when runtime budget baseline thresholds drift or are exceeded.
- AC-3: Selector policy keeps heavy compatibility run-mode commands out of fast-gate and ci-tools fast mode.
- AC-4: Docs/runbook marker parity remains deterministic, including remediation marker coverage for every checker reason code.
- AC-5: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | checker + valid dry-run reports | checker returns `status=pass`, `final_decision=GO`, deterministic markers |
| C-02 | AC-1 | Regression | checker + tampered compatibility report marker | checker returns `NO-GO` with deterministic report-contract drift reason |
| C-03 | AC-2 | Unit | checker + threshold fixture | threshold keys parse deterministically; missing/invalid threshold fails closed |
| C-04 | AC-3 | Integration | checker + `scripts/ci/test_ci_tools.sh` + `.github/workflows/ci-fast-gate.yml` | required fast-mode contract entry present; heavy run-mode command leakage rejected |
| C-05 | AC-4 | Regression | checker + docs/runbook marker drift fixture | checker fails closed on parity/remediation marker drift |
| C-06 | AC-5 | Performance | checker + valid baseline inputs | checker runtime stays within threshold and reports deterministic budget markers |

## Test Mapping
- `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract unit_compatibility_ci_dry_run_checker_accepts_valid_reports -- --exact`
- `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract functional_compatibility_ci_dry_run_checker_rejects_tampered_report_contract -- --exact`
- `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract integration_compatibility_ci_dry_run_checker_enforces_selector_and_workflow_exclusion -- --exact`
- `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract regression_compatibility_ci_dry_run_checker_rejects_docs_remediation_parity_drift -- --exact`
- `cargo test -p kamn-core --test compatibility_ci_dry_run_governance_contract performance_compatibility_ci_dry_run_checker_stays_within_budget -- --exact`

## Success Metrics
- Compatibility dry-run governance contracts fail closed deterministically under marker/schema/threshold drift.
- Fast-gate/local-heavy boundary remains explicit and test-backed.
- Strategy + ops docs remain synchronized with checker taxonomy, thresholds, and remediation markers.
