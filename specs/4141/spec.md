# Issue #4141 Spec

- Title: Subtask: add ci smoke checker for concurrency marker lineage and local-heavy lane exclusions
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
The CI-smoke concurrency policy path must prove marker lineage and explicit local-heavy exclusion behavior without running expensive deep lanes.

## Acceptance Criteria
- AC-1: Checker fails closed when marker lineage/taxonomy fields drift.
- AC-2: Fast-gate selector output keeps heavy concurrency/local-heavy paths deterministic and default-excluded.
- AC-3: CI smoke path remains bounded and low-cost.

## Scope
In scope:
- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/ci/select_targets.sh`
- `scripts/ci/test_select_targets.sh`
- `specs/4141/{spec.md,plan.md,tasks.md}`

Out of scope:
- Enabling deep concurrency lanes in PR fast-gate.
- Production runtime logic changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | Tampered report paths in `test_check_invariant_fuzz_concurrency_policy.sh` | Deterministic mismatch reason codes + `NO-GO` fail-closed decision |
| C-02 | AC-2 | Integration | `bash scripts/ci/test_select_targets.sh` | Selector keeps local-heavy lanes opt-in/local-only with deterministic scope markers |
| C-03 | AC-3 | Functional | `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh` | CI smoke contract lane stays bounded and emits low-cost boundary markers |

## Test Mapping
- `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `bash scripts/ci/test_select_targets.sh`

## Success Metrics
- Marker-lineage drift always fails closed.
- Local-heavy concurrency execution remains default-excluded from fast-gate.
- No new shell wrappers/scripts required for closure.
