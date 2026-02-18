# Spec — Issue #3971

- Title: Subtask: add wrapper dispatch parity harness and legacy entrypoint compatibility assertions
- Parent: #3966
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Wrapper matrix tests verify symlink/manifest resolution, but there is no unified parity harness asserting deterministic fallback reason markers for unknown legacy wrappers. This leaves compatibility failure signaling partially uncontracted.

## Objective

Add a dedicated wrapper-dispatch parity harness that executes legacy-wrapper matrix suites and asserts deterministic unknown-wrapper fallback markers, then wire/document it in CI contract surfaces.

## Scope

In scope:
- Add CI harness script that runs non-Kolme wrapper matrix suites.
- Assert deterministic unknown-wrapper fallback marker taxonomy in harness.
- Wire harness into `scripts/ci/test_ci_tools.sh` fast regression lane.
- Update CI strategy marker docs/contracts for new harness + markers.

Out of scope:
- Refactoring dispatcher implementation.
- Adding new wrapper families.

## Acceptance Criteria

- AC-1: Parity harness executes legacy-wrapper matrix suites and fails closed if any suite fails.
- AC-2: Harness asserts deterministic unknown-wrapper fallback markers:
  - `fallback_reason_taxonomy_version=kamn.framework.non-kolme-dispatch-fallback-reason-taxonomy.v1`
  - `fallback_reason_codes_csv=dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped`
  - `fallback_reason_code=dispatcher_unknown_wrapper`
- AC-3: CI tools fast regression includes harness execution.
- AC-4: CI strategy docs and marker contract tests include harness command/marker references.

## Conformance Cases

- C-01 (AC-1): Harness runs core matrix scripts and reports pass marker.
- C-02 (AC-2): Unknown-wrapper dispatch probe emits deterministic fallback marker set.
- C-03 (AC-3): `test_ci_tools.sh` executes the new harness in fast mode.
- C-04 (AC-4): `test_ci_strategy_contract.sh` enforces doc markers for harness and fallback taxonomy codes.

## Success Metrics

- Legacy-wrapper compatibility failures become deterministic and machine-parseable.
- Wrapper parity harness remains fast and stable in CI tools lane.
