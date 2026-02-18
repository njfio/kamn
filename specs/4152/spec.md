# Issue #4152 Spec

- Title: Subtask: add red tests for rollback trigger mismatch and deterministic reason-code taxonomy
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md

## Problem Statement
Rollback trigger governance needs explicit mismatch and taxonomy-drift regression tests so policy failures stay deterministic and fail closed.

## Acceptance Criteria
- AC-1: Rollback trigger mismatch fixtures fail policy validation with deterministic NO-GO reason markers.
- AC-2: Reason taxonomy drift fixtures fail policy validation with deterministic mismatch markers.
- AC-3: Repeated rollback mismatch evaluations preserve deterministic reason-code ordering.

## Scope
In scope:
- Add rollback mismatch and taxonomy drift coverage in governance rollback shell tests.
- Add deterministic repeated-run assertions for mismatch reason ordering.
- Lifecycle artifacts for issue `#4152`.

Out of scope:
- Policy checker behavior redesign.
- New runtime lane features.

## Shell-Surface Impact Estimates
- shell_loc_delta_estimate: 80
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: 0.0000
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Tampered rollback trigger projection in rollback policy report | Policy checker rejects with deterministic mismatch markers and `NO-GO` outcome |
| C-02 | AC-2 | Regression | Tampered rollback reason taxonomy version/codes CSV | Policy checker rejects with deterministic taxonomy mismatch marker |
| C-03 | AC-3 | Regression | Execute identical rollback mismatch check twice | `decision_reasons` ordering remains identical between runs |

## Test Mapping
- `bash scripts/governance/test_check_governance_lifecycle_rollback_policy.sh`

## Success Metrics
- `#4152` closes with deterministic rollback mismatch/taxonomy drift coverage added and passing in governance rollback policy tests.
