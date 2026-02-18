# Spec - Issue #3970

- Title: Subtask: create wrapper-family migration matrix and convert first high-LOC family to shared manifest implementation
- Parent: #3966
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

The non-Kolme wave migration fixtures exist, but the first-family conversion (wave-1 canary wrappers) is not explicitly asserted by the shared dispatcher wrapper-matrix contract. This leaves room for silent regression where canary entrypoints drift away from shared manifest dispatch without immediate fast-gate failure.

## Objective

Make wave-1 conversion contractual by ensuring canary wrappers are covered in the non-Kolme dispatcher parity matrix and keep migration-matrix artifacts tied to deterministic baseline checks.

## Scope

In scope:
- Add explicit canary wrapper assertions to `scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh`.
- Keep deterministic wave-1 matrix/baseline fixtures as the source for first-family migration verification.
- Add issue lifecycle artifacts (`spec.md`, `plan.md`, `tasks.md`) with AC->conformance mapping.

Out of scope:
- Converting all remaining wrapper families in this subtask.
- Refactoring dispatcher runtime behavior.

## Acceptance Criteria

- AC-1: Migration matrix artifacts for non-Kolme wave-1 remain deterministic and versioned.
- AC-2: First-family conversion (canary wrappers) is explicitly asserted as shared-dispatch symlink entrypoints with manifest resolution checks.
- AC-3: Unit/Functional/Integration/Regression coverage for this contract lane is present and passing (or justified N/A).

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_non_kolme_wave1_wrapper_family_baseline_contract.sh` passes and validates matrix/baseline determinism.
- C-02 (AC-2): `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh` verifies canary + governance wrappers are symlink-backed and manifest-resolvable via shared dispatcher.
- C-03 (AC-2): Unknown-wrapper resolution in dispatcher matrix test fails closed.
- C-04 (AC-3): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes with wrapper/migration contract lanes included.

## Success Metrics

- Any canary wrapper drift from shared dispatcher fails fast in the wrapper matrix contract lane.
- Wave-1 migration matrix and baseline fixtures remain deterministic under generated/check flows.
