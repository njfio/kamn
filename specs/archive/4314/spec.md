# Issue #4314 Spec

- Title: `Task: implement durable block commit persistence with digest-finality fail-closed validation governance`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4310`

## Problem Statement
Durable block commit promotion evidence needs deterministic replay/tamper rejection and stable reason projection with explicit CI-smoke vs local-heavy boundary governance.

## Scope
In:
- Persisted block commit digest/finality mismatch and tamper rejection conformance matrix.
- Durable commit checker reason projection taxonomy and CI/local-heavy lane boundary enforcement.
- Release go/no-go and CI strategy docs markers for durable commit governance.

Out:
- Consensus algorithm replacement.
- Always-on heavy validation in CI.

## Acceptance Criteria
- AC-1: commit artifacts persist and replay validation fails closed on digest/finality/tamper drift.
- AC-2: durable commit checker reason projection taxonomy is deterministic and checker-consumable.
- AC-3: CI-smoke and local-heavy lane boundaries are enforced with deterministic reason markers.
- AC-4: docs contract markers for release checklist and CI strategy remain parity-guarded by tests.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix unit_replay_rejects_persisted_payload_digest_mismatch_reason_code -- --exact` | digest mismatch replay path fails closed with deterministic reason |
| C-02 | AC-1 | Functional | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix functional_replay_rejects_persisted_checkpoint_missing_reason_code -- --exact` | missing checkpoint fails closed with deterministic reason |
| C-03 | AC-2 | Unit | `cargo test -p kamn-core --test block_commit_checker_reason_mapping unit_replay_drift_reason_projection_is_deterministic -- --exact` | reason projection class/taxonomy markers are deterministic |
| C-04 | AC-3 | Integration | `cargo test -p kamn-core --test block_commit_checker_reason_mapping integration_checker_projection_and_lane_boundary_contracts_are_consistent -- --exact` | projection and lane-boundary contracts compose deterministically |
| C-05 | AC-1 | Regression | `cargo test -p kamn-core --test block_commit_persistence_tamper_matrix regression_replay_height_mismatch_reason_code_stable -- --exact` | replay mismatch reason remains stable |
| C-06 | AC-3 | Regression | `cargo test -p kamn-core --test block_commit_checker_reason_mapping regression_local_heavy_opt_in_reason_code_stays_stable -- --exact` | local-heavy opt-in boundary reason remains stable |
| C-07 | AC-2 | Performance | `cargo test -p kamn-core --test block_commit_checker_reason_mapping performance_reason_projection_and_boundary_loops_stay_within_local_budget -- --exact` | projection/boundary loops remain bounded |
| C-08 | AC-4 | Docs | `cargo test -p kamn-core --test release_gonogo_checklist_docs checklist_contains_block_commit_persistence_mismatch_tamper_gate -- --exact` | release checklist markers remain present |
| C-09 | AC-4 | Docs | `cargo test -p kamn-core --test ci_strategy_docs doc_contains_durable_commit_checker_ci_boundary_markers -- --exact` | CI strategy markers remain present |

## Test Mapping
- `crates/kamn-core/tests/block_commit_persistence_tamper_matrix.rs`
- `crates/kamn-core/tests/block_commit_checker_reason_mapping.rs`
- `crates/kamn-core/src/block_pipeline.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Success Metrics
- Replay tamper cases fail closed with deterministic reason codes.
- Durable commit checker reason projection and lane-boundary enforcement are deterministic.
- Docs parity tests prevent release/CI marker drift.
