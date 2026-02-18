# Issue #4977 Spec

- Title: Subtask: integrate ceiling+ratio ratchet checks into ci-fast-gate required status checks
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4977 requires CI fast-gate to execute shell LOC hard-ceiling and shell:Rust ratio guardrail checks as deterministic merge-blocking policy checks.

## Acceptance Criteria
- AC-1: `ci-fast-gate` includes both required steps:
  - `Check shell-rust ratio guardrail`
  - `Check shell LOC hard ceiling`
  with their checker commands and JSON output wiring.
- AC-2: Both checkers fail closed on threshold violations with deterministic reason-taxonomy markers.
- AC-3: Wiring, checker contracts, and CI command-surface coverage tests pass.
- AC-4: Issue/process/spec artifacts are synchronized to implemented state.

## Scope
In scope:
- CI fast-gate wiring validation for ceiling + ratio checks.
- Checker contract validation and deterministic fail behavior evidence.
- Spec/task/plan lifecycle closure for issue #4977.

Out of scope:
- Changes to threshold values.
- New policy dimensions outside ratio + hard ceiling integration.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | Inspect `ci-fast-gate` workflow wiring | Both required step names + commands + output paths exist |
| C-02 | AC-2 | Regression | Force ratio and ceiling threshold violations | Both checkers exit non-zero and emit deterministic reason codes |
| C-03 | AC-3 | Unit/Functional | Run scoped checker + wiring contract tests | All scoped tests pass |
| C-04 | AC-4 | Functional/Regression | Verify issue + spec/task/plan lifecycle markers | Issue/process/spec state is consistent with implementation |

## Test Mapping
- AC-1:
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- AC-2:
  - `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file <tmp-fail> --output-json <tmp>`
  - `bash scripts/ci/check_shell_loc_hard_ceiling.sh --ceiling-file <tmp-fail> --output-json <tmp>`
- AC-3:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
  - `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `specs/4977/spec.md`
  - `specs/4977/plan.md`
  - `specs/4977/tasks.md`

## Success Metrics
- All ACs map to passing conformance cases with deterministic evidence.
- Fast-gate required shell ceiling + ratio checks are verified in workflow contract coverage.
