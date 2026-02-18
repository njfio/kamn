# Issue #4976 Spec

- Title: Subtask: add hard shell LOC ceiling threshold fixture and checker red-green tests
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4976 requires deterministic hard shell LOC ceiling fixture/checker behavior with explicit red/green test coverage and CI wiring validation.

## Acceptance Criteria
- AC-1: Hard shell LOC ceiling fixture exists and checker reads it deterministically.
- AC-2: Checker fails closed on ceiling exceedance with deterministic reason taxonomy markers.
- AC-3: Unit/functional/integration/regression tests for fixture/checker and fast-gate wiring pass.
- AC-4: Issue/process/spec artifacts are synchronized to implemented state.

## Scope
In scope:
- `.ci/shell-loc-hard-ceiling.env` fixture usage.
- `scripts/ci/check_shell_loc_hard_ceiling.sh` behavior validation.
- `scripts/ci/test_check_shell_loc_hard_ceiling.sh` and wiring contract evidence.

Out of scope:
- Threshold policy redesign beyond the hard ceiling checker contract.
- Unrelated shell-surface governance features.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run checker with `.ci/shell-loc-hard-ceiling.env` | Exit `0` and deterministic markers |
| C-02 | AC-2 | Regression | Run checker with intentionally low temporary ceiling | Exit non-zero with `shell_loc_hard_ceiling_exceeded` |
| C-03 | AC-3 | Unit/Integration | Run fixture/checker + fast-gate wiring tests | All scoped suites pass |
| C-04 | AC-4 | Functional/Regression | Verify issue/spec lifecycle state | Issue/spec/tasks reflect Implemented completion |

## Test Mapping
- AC-1:
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file .ci/shell-loc-hard-ceiling.env --output-json <tmp>`
- AC-2:
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file <tmp HARD_SHELL_LOC_MAX=30000> --output-json <tmp>`
- AC-3:
  - `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `specs/4976/spec.md`
  - `specs/4976/plan.md`
  - `specs/4976/tasks.md`

## Success Metrics
- All ACs are mapped to deterministic conformance cases and passing test evidence.
- Hard ceiling checker fixture usage and fail-closed behavior are explicitly evidenced.
