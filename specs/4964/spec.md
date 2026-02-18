# Issue #4964 Spec

- Title: Task: implement hard shell LOC ceiling policy checker with deterministic reason taxonomy
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
The milestone needed a fail-closed hard shell LOC ceiling checker with deterministic machine-readable reason codes and report outputs.

## Acceptance Criteria
- AC-1: Checker evaluates current shell LOC against configured hard ceiling.
- AC-2: Ceiling violations fail with deterministic reason markers.
- AC-3: Checker emits machine-readable JSON report payload.
- AC-4: Hard-ceiling contract tests pass.

## Scope
In scope:
- Hard-ceiling checker contract lifecycle and evidence finalization.
- Deterministic reason taxonomy/report behavior validation.

Out of scope:
- CI wiring of checker as required gate (covered by #4965).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | shell LOC measurement vs threshold fixture | pass/fail evaluated deterministically |
| C-02 | AC-2 | Regression | threshold exceedance mutation | deterministic NO-GO reason markers |
| C-03 | AC-3 | Unit | checker output JSON parsing | schema/metrics fields present |
| C-04 | AC-4 | Regression | `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh` | contract suite passes |

## Test Mapping
- AC-1/AC-2/AC-3/AC-4: `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`

## Success Metrics
- Hard-ceiling checker behavior is deterministic and covered by contract tests.
