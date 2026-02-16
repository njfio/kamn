# Issue #4322 Spec

- Title: `Subtask: implement durable commit checker reason mapping with ci smoke and local-heavy boundary enforcement`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4314`

## Problem Statement
Durable block commit promotion checks need deterministic reason projection and explicit ci-smoke/local-heavy boundary enforcement so fast lanes stay low-cost while deep validation remains opt-in.

## Scope
In:
- Add durable commit checker reason projection API with deterministic taxonomy markers.
- Add durable commit checker lane-boundary contract API for ci-smoke vs local-heavy opt-in enforcement.
- Add conformance tests (unit/functional/integration/regression/performance).
- Update `docs/ci/strategy.md` with durable commit checker smoke/local-heavy boundary markers and commands.

Out:
- New shell wrapper families or workflow lane expansion.
- Consensus algorithm/storage schema redesign.

## Acceptance Criteria
- AC-1: durable commit checker reason mapping is deterministic and stable.
- AC-2: ci-smoke checker boundary remains low-cost and explicit.
- AC-3: local-heavy lane remains explicit opt-in and fails closed on boundary misuse.
- AC-4: ci strategy docs describe the durable commit checker boundary contract and markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-core --test block_commit_checker_reason_mapping unit_replay_drift_reason_projection_is_deterministic -- --exact` | canonical replay drift maps to deterministic reason class/code/source markers |
| C-02 | AC-2 | Functional | `cargo test -p kamn-core --test block_commit_checker_reason_mapping functional_ci_smoke_lane_boundary_emits_low_cost_markers -- --exact` | ci-smoke boundary emits stable low-cost markers |
| C-03 | AC-3 | Integration | `cargo test -p kamn-core --test block_commit_checker_reason_mapping integration_checker_projection_and_lane_boundary_contracts_are_consistent -- --exact` | reason projection and lane-boundary enforcement remain coherent |
| C-04 | AC-3 | Regression | `cargo test -p kamn-core --test block_commit_checker_reason_mapping regression_local_heavy_opt_in_reason_code_stays_stable -- --exact` | local-heavy without opt-in fails closed with stable reason code |
| C-05 | AC-2 | Performance | `cargo test -p kamn-core --test block_commit_checker_reason_mapping performance_reason_projection_and_boundary_loops_stay_within_local_budget -- --exact` | projection + boundary loops remain bounded |
| C-06 | AC-4 | Docs | `cargo test -p kamn-core --test ci_strategy_docs doc_contains_durable_commit_checker_ci_boundary_markers -- --exact` | ci strategy doc contains durable commit checker boundary commands and markers |

## Test Mapping
- `crates/kamn-core/tests/block_commit_checker_reason_mapping.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`

## Success Metrics
- Durable commit checker reason projection outputs are deterministic and taxonomy-versioned.
- ci-smoke and local-heavy boundary misuse fails closed with stable reason codes.
- CI strategy docs are test-guarded for boundary marker drift.
