# Issue #4134 Spec

- Title: Task: add concurrency stress lanes and race-safety contract checks with ci-local budget controls
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Concurrency stress assurance requires deterministic checker contracts and explicit CI/local-heavy boundaries so race-safety evidence stays fail-closed without forcing expensive deep lanes into fast-gate.

## Acceptance Criteria
- AC-1: Concurrency marker contracts are validated by deterministic checker lanes.
- AC-2: Local-heavy stress lanes are excluded from fast-gate by default and documented.
- AC-3: Drift checks fail closed on missing or mismatched markers/taxonomy fields.
- AC-4: Unit/Functional/Integration/Regression coverage for this lane family passes.

## Scope
In scope:
- Runtime invariant/fuzz/concurrency contract lane + policy checker scripts/tests.
- CI selector routing assertions for deterministic local-heavy exclusion behavior.
- CI/docs marker contracts for concurrency boundary governance.
- `specs/4134/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Always-on deep concurrency stress in merge-gate.
- Production runtime algorithm changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh` | Contract lane emits deterministic pass markers and report schema values |
| C-02 | AC-2 | Integration | `bash scripts/ci/test_select_targets.sh` | Local-heavy lane routing remains selector-gated and excluded by default |
| C-03 | AC-3 | Regression | `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh` | Marker/taxonomy drift paths fail closed with deterministic reason codes |
| C-04 | AC-4 | Conformance | Targeted runtime + CI selector checks | All required lane-policy checks remain green |

## Test Mapping
- `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `bash scripts/ci/test_select_targets.sh`

## Success Metrics
- Deterministic CI-smoke concurrency checker path remains fail-closed on drift.
- Local-heavy lane routing remains opt-in/default-excluded in selector outputs.
- Shell-surface ratio impact for this closure pass remains neutral.
