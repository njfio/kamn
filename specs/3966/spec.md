# Spec - Issue #3966

- Title: Task: inventory and migrate prioritized high-LOC wrapper families to manifest shared implementations
- Parent: #3964
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

High-LOC wrapper families were historically duplicated across shell entrypoints, increasing maintenance cost and drift risk in contract-lane execution.

## Objective

Inventory and migrate prioritized wrapper families to shared manifest dispatch paths while preserving legacy entrypoint compatibility and parity regressions in CI.

## Scope

In scope:
- Wave-1 first-family migration matrix and conversion assertions (subtask `#3970`).
- Wrapper dispatch parity + legacy entrypoint compatibility harness with deterministic fallback markers (subtask `#3971`).
- Fast-gate CI coverage and documentation marker updates for shared-dispatch governance.

Out of scope:
- Full elimination of all shell entrypoints in one task.
- Protocol/runtime feature changes unrelated to dispatch and wrapper governance.

## Acceptance Criteria

- AC-1: Target wrapper-family migrations to shared manifest/dispatcher paths are asserted by deterministic matrix checks.
- AC-2: Legacy wrapper entrypoint compatibility remains fail-closed and deterministic under dispatcher routing.
- AC-3: Regression checks cover parity, command-surface, and strategy-marker drift.
- AC-4: Unit/Functional/Integration/Regression coverage for this task scope is present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh` and `bash scripts/ci/test_non_kolme_wave1_wrapper_family_baseline_contract.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh` passes and emits deterministic fallback markers.
- C-03 (AC-3): `bash scripts/ci/test_ci_strategy_contract.sh` and `bash scripts/ci/test_ci_tools_command_surface_contract.sh` pass.
- C-04 (AC-4): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes with wrapper-family and compatibility lanes included.

## Success Metrics

- First-family non-Kolme migration and wrapper parity checks fail fast on drift.
- Shared-dispatch compatibility markers remain deterministic and machine-parsable in CI fast gate.
