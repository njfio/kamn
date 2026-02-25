# Spec: Issue #5939 - Task: Expand mutation and coverage gates (llvm-cov) for critical runtime/security paths

- Issue: #5939
- Status: Implemented
- Type: task
- Priority: P1
- Area: qa
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5920

## Problem Statement
Mutation and coverage signals are currently too narrow for risk profile.

## Scope
In scope:
- Broaden mutation scope and add enforceable coverage reporting for critical crates.

Out of scope:
- Blind global coverage percentage targets without path criticality weighting.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Mutation testing includes runtime/API/network/security paths with tracked escape remediation.
- AC-2: Coverage is reported in CI and guarded against regressions on critical modules.
- AC-3: PRs include clear mutation/coverage evidence aligned to changed critical code.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Mutation testing includes runtime/API/network/security paths with tracked escape remediation.
- C-02 (Functional, AC-2): Verify Coverage is reported in CI and guarded against regressions on critical modules.
- C-03 (Functional, AC-3): Verify PRs include clear mutation/coverage evidence aligned to changed critical code.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.


## Required Test Categories
- Unit: mutation helper/tooling tests
- Functional: critical-path suite runs under mutation
- Integration: CI coverage artifact generation and checks
- Regression: escaped mutants tracked/fixed
- Performance: mutation lane budget enforced

## Dependencies
- #5920

## Implementation Notes
- Added critical-path coverage thresholds: `.ci/critical-path-coverage-thresholds.json`.
- Added fail-closed coverage policy checker + runner:
  - `scripts/ci/check_critical_path_coverage.py`
  - `scripts/ci/run_critical_path_coverage_gate.sh`
- Added fail-closed bounded mutation runner:
  - `scripts/ci/run_critical_path_mutation_gate.sh`
- Added CI-tool regression tests:
  - `scripts/ci/test_check_critical_path_coverage.sh`
  - `scripts/ci/test_run_critical_path_mutation_gate.sh`
- Wired workspace pre-merge CI gate + report artifacts in `.github/workflows/ci-fast-gate.yml`.
- Added PR evidence section to `.github/pull_request_template.md` for mutation/coverage reporting.
- Added architecture decision record:
  - `docs/architecture/adr-critical-path-assurance-gates.md`

## Verification Snapshot
- Mutation gate: `slice_count=6`, `tested_mutants=10`, `caught_mutants=10`, `missed=0`, `unviable=0`, `timeout=0`.
- Coverage gate: `target_count=6`, `failed_targets=0`, `final_decision=GO`.
