# Spec: Issue #4457

Status: Implemented
Issue: #4457
Parent: #4449
Milestone: R27.39 Runtime decomposition, panic-free execution, and dependency-license governance
Priority: P1

## Problem Statement

Dependency/license metadata governance needs deterministic reason taxonomy outputs and explicit CI
smoke versus local-heavy execution boundaries. Without those contracts, auditability and CI cost
control can drift.

## Scope

In scope:
- Deterministic reason-taxonomy outputs for workspace license metadata governance checks.
- CI smoke/local-heavy boundary markers and fail-closed local-heavy opt-in enforcement.
- CI strategy documentation updates and docs-contract coverage for the governance boundary matrix.

Out of scope:
- Deep metadata audit lanes in CI fast-gate.
- New external license policy processes.

## Acceptance Criteria

AC-1:
Given workspace license governance checks, when pass/fail scenarios execute, then deterministic
`reason_taxonomy_version`, `reason_codes_csv`, `reason_codes_value`, and `reason_class` markers are
emitted.

AC-2:
Given CI smoke versus local-heavy execution modes, when lane boundaries are evaluated, then
`ci_smoke_local_heavy_boundary_status`, `ci_smoke_lane_cost_profile`, and
`local_heavy_lane_execution_mode` markers enforce explicit local-heavy opt-in.

AC-3:
Given CI strategy docs contracts, when docs tests run, then metadata-governance taxonomy and
CI/local boundary matrix markers are present and fail closed on drift.

## Conformance Cases

- C-01 (AC-1, Functional/Conformance):
  - Test: `bash scripts/ci/test_check_workspace_license_policy.sh`
  - Expectation: deterministic taxonomy/value/class outputs for pass/fail paths.

- C-02 (AC-2, Integration/Conformance):
  - Test: `bash scripts/ci/test_check_workspace_license_policy.sh`
  - Expectation: CI-smoke/local-heavy boundary markers are emitted; local-heavy mode requires
    explicit opt-in.

- C-03 (AC-3, Regression/Conformance):
  - Test: `cargo test -p kamn-core --test ci_strategy_docs`
  - Expectation: docs include metadata-governance taxonomy and CI/local boundary matrix markers.

## Success Metrics / Observable Signals

- Workspace license checker emits stable machine-readable governance markers.
- Local-heavy execution without opt-in fails closed with deterministic reason code.
- Docs contract fails if CI strategy governance matrix markers drift.
