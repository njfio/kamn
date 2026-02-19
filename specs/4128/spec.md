# Issue #4128 Spec

- Title: Epic: R27.18 close advanced validation depth and deterministic assurance gaps
- Status: Implemented
- Type: epic
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Validation depth needed deterministic property, fuzz, and concurrency governance with bounded CI-smoke cost and explicit local-heavy boundaries.

## Acceptance Criteria
- AC-1: Property-based tests enforce core transition invariants with deterministic seeds.
- AC-2: Fuzz and concurrency contracts validate parser/race behavior with fail-closed markers.
- AC-3: CI fast-gate remains low-cost while heavy validation lanes remain explicit local opt-in.

## Scope
In scope:
- Story chain closeout: `#4129` and `#4130` (and child tasks/subtasks).
- R27.18 marker-governance docs and contract checks.
- `specs/4128/{spec.md,plan.md,tasks.md}`.

Out of scope:
- External OSS-Fuzz platform onboarding.
- Always-on deep stress in merge-gate.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Property invariant suites (`property_invariant_helpers_contracts`, task/escrow/peer property tests) | Deterministic seed-backed invariant checks pass |
| C-02 | AC-2 | Regression | Fuzz + concurrency contract suites (`cargo_fuzz_target_contract`, concurrency policy/lane tests) | Drift/tamper paths fail closed with deterministic markers |
| C-03 | AC-3 | Integration | CI selector + docs marker contracts | CI-smoke bounded paths remain default, local-heavy paths remain opt-in/local-only |

## Test Mapping
- `cargo test -p kamn-core --test property_invariant_helpers_contracts`
- `cargo test -p kamn-core --test task_escrow_suite_discovery_parallel_contract`
- `cargo test -p kamn-core --test cargo_fuzz_target_contract`
- `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `bash scripts/ci/test_select_targets.sh`

## Success Metrics
- Property/fuzz/concurrency governance chain is fully specified and implemented.
- CI-smoke versus local-heavy boundaries remain explicit and deterministic.
- Shell-surface delta for closure-only backfill remains neutral.
