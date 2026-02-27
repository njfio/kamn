# Spec: Issue #6141 - Task: [X-03] Run fuzz targets in CI by default policy lane

- Issue: #6141
- Status: Reviewed
- Type: task
- Priority: P1
- Area: qa
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6102

## Problem Statement
The coverage-guided parser fuzz contract lane exists (`run_input_mutation_coverage_guided_contract_lane.sh`) but is not explicitly enforced in the default fast-gate path for Rust changes. R59 `X-03` requires deterministic CI execution with evidence artifacts.

## Scope
In scope:
- Add explicit `ci-fast-gate` execution of the coverage-guided parser fuzz contract lane for Rust-scoped PRs.
- Upload deterministic fuzz-lane evidence JSON from CI.
- Add workflow policy regression checks that fail if this lane/report wiring is removed.

Out of scope:
- Enabling deep coverage-guided fuzz lane in fast gate.
- Redesigning target selection logic in `scripts/ci/select_targets.sh`.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `ci-fast-gate` runs `run_input_mutation_coverage_guided_contract_lane.sh` for Rust-scoped runs.
- AC-2: The coverage-guided fuzz lane emits and uploads a deterministic JSON report artifact in fast gate.
- AC-3: Workflow scope policy tests fail closed if the fast-gate fuzz lane or artifact wiring regresses.

## Conformance Cases
- C-01 (AC-1, Conformance): Workflow contains explicit runtime coverage-guided parser fuzz contract lane step.
- C-02 (AC-2, Functional/Conformance): Workflow contains explicit upload step for `runtime-input-mutation-coverage-guided-contract-report.json`.
- C-03 (AC-3, Regression): `scripts/ci/test_workflow_scope_policy.sh` asserts presence of lane/report wiring and continues asserting deep-lane exclusion.

## Success Metrics / Observable Signals
- Fast gate executes bounded coverage-guided fuzz lane on Rust PRs.
- CI artifacts include `runtime-input-mutation-coverage-guided-contract-report.json`.
- Scope-policy regression test passes post-change and would fail if the lane wiring is removed.
