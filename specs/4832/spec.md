# Spec — Issue #4832

- Title: Subtask: wire ratio and script-budget checks into CI fast gate with bounded runtime budgets
- Parent: Parent task: #4817
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Wire shell-surface ratio and script-budget governance checks directly into `ci-fast-gate` so every scoped run enforces deterministic fail-closed policy contracts.

## Problem Statement

Script-surface budget checks existed in fast gate, but combined shell-surface ratio policy checks and deterministic shell-vs-Rust telemetry were not enforced directly in workflow execution.

## Scope

In scope:
- add fast-gate workflow steps for:
  - combined shell-surface trend report generation
  - combined shell-surface policy enforcement
  - shell-vs-Rust telemetry collection
- add workflow artifact uploads for ratio/policy/telemetry outputs
- ensure shell-surface checks remain behind `run_script_surface_budget_checks` selector gate
- add deterministic workflow-wiring contract test and include it in CI tool regression lane
- keep runtime bounded under existing fast-gate budget contract (`timeout-minutes: 20`)

Out of scope:
- threshold taxonomy redesign (existing policy checker contract retained)
- non-fast-gate workflow changes

## Acceptance Criteria

- AC-1: Fast-gate workflow runs combined shell-surface ratio report + policy checks with deterministic fail-closed behavior.
- AC-2: Fast-gate workflow emits shell-vs-Rust telemetry and uploads ratio/policy/telemetry artifacts.
- AC-3: Workflow checks remain selector-gated and preserve bounded fast-gate runtime budget.

## Conformance Cases

- C-01 (AC-1, Functional): `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh` verifies workflow includes ratio generation/policy check commands and threshold fixture wiring.
- C-02 (AC-2, Conformance): `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh` verifies telemetry collector output contracts used by fast-gate workflow.
- C-03 (AC-3, Integration/Regression): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes with new workflow-wiring contract test and budget-gated command surface.

## Success Metrics / Signals

- `ci-fast-gate` includes deterministic ratio + script-budget governance commands under selector gate.
- Workflow uploads `ci-combined-shell-surface-trend-*` and `ci-shell-rust-loc-telemetry-*` artifacts.
- CI regression suite remains green with new workflow wiring contract test.
