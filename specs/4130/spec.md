# Issue #4130 Spec

- Title: Story: add fuzzing and concurrency stress governance with low-cost ci and local-heavy boundaries
- Status: Implemented
- Type: story
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Parser fuzzing and concurrency stress assurance needed deterministic marker-governance contracts that catch drift while keeping fast-gate cost bounded.

## Acceptance Criteria
- AC-1: Fuzz and concurrency markers are deterministic and policy-validated.
- AC-2: CI smoke checks cover contract drift without running heavy lanes.
- AC-3: Unit/Functional/Integration/Regression tests for this surface pass.
- AC-4: Local-heavy validation paths are explicit, documented, and opt-in.

## Scope
In scope:
- Child task chain: `#4133` (deterministic fuzz governance) and `#4134` (concurrency stress contracts).
- Parser seed/provenance and concurrency policy/checker contracts.
- CI selector boundary/exclusion rules and docs marker contracts.
- `specs/4130/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Always-on exhaustive fuzzing or deep stress in merge-gate.
- Distributed chaos-engineering rollout.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `cargo test -p kamn-core --test cargo_fuzz_target_contract` | Deterministic fuzz seed/provenance markers are enforced |
| C-02 | AC-2 | Regression | runtime concurrency policy/lane tests | Drift/tamper fails closed without requiring deep-lane execution |
| C-03 | AC-3 | Integration | CI selector matrix | Routing for affected surfaces remains deterministic and green |
| C-04 | AC-4 | Conformance | `ci_strategy_docs` invariant-fuzz-concurrency marker test | CI-smoke/local-heavy boundary markers remain explicit |

## Test Mapping
- `cargo test -p kamn-core --test cargo_fuzz_target_contract`
- `cargo test -p kamn-core --test ci_strategy_docs -- doc_contains_invariant_fuzz_concurrency_ci_smoke_boundary_contract_markers --exact`
- `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `bash scripts/ci/test_select_targets.sh`

## Success Metrics
- Deterministic marker-governance contracts are present for fuzz + concurrency paths.
- CI-smoke remains bounded while local-heavy paths remain explicit opt-in.
- Story closure is fully backed by spec artifacts.
