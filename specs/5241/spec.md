# Issue #5241 Spec

- Title: Subtask: offset #4096 shell LOC increase via overload checker test compaction
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Issue #4096 introduced a bounded shell LOC increase for checker coverage. Guardrails remain green, but shell ratio movement regressed and should be offset in-milestone.

## Acceptance Criteria
- AC-1: Shell LOC introduced by overload checker regression surface is reduced materially without losing behavior coverage.
- AC-2: Overload checker regression behavior remains deterministic and fail-closed across pass/fail cases.
- AC-3: Existing command-surface contracts continue passing after compaction.
- AC-4: Shell-rust ratio and shell LOC hard-ceiling checks remain green.

## Scope
In scope:
- Compact overload checker regression test shell surface by moving test logic to Python.
- Keep `scripts/ci/test_check_daemon_os_signal_stress_policy.sh` as command-surface compatible wrapper.
- Add Python regression test implementation and preserve existing pass/fail assertions.
- Update/verify CI contract tests that assert command-surface expectations.

Out of scope:
- Checker behavior redesign.
- New CI workflows.
- Runtime feature changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: -95
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: -0.0006
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | shell wrapper + python test split | shell LOC decreases while command surface remains stable |
| C-02 | AC-2 | Functional | overload checker pass/fail fixture scenarios | deterministic pass/fail reason-code assertions preserved |
| C-03 | AC-3 | Integration | ci tools command-surface contract test | required overload test command remains present |
| C-04 | AC-4 | Conformance | shell ratio and hard-ceiling guard tests | both checks pass |

## Test Mapping
- C-01 -> `git diff --numstat origin/main...HEAD`
- C-02 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-03 -> `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- C-04 -> `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh` and `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`

## Success Metrics
- Net shell LOC decreases relative to pre-mitigation branch state.
- Overload checker regression guarantees remain intact.
