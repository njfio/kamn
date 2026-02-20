# Issue #5265 Spec

- Title: Task: implement M1 batch scheduler thresholds and merkle-batch persistence execution path
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M1 exposes deterministic merkle assembly and anchoring contracts, but there is no operational scheduling contract for count/window triggers and no PostgreSQL execution path to persist merkle batch lifecycle state updates.

## Scope
In:
- Add deterministic M1 scheduler trigger evaluation for `message-count` and `window-seconds` policies.
- Add PostgreSQL execution-adapter operations to persist merkle batch creation, message-to-batch assignment, and submission/confirmation state transitions.
- Add fail-closed validation for invalid batch identifiers and invalid transition payloads.
- Add tests covering scheduler determinism and live adapter persistence behavior.

Out:
- Background daemon orchestration loops.
- Cross-cluster anchoring coordination.
- New shell/python/workflow tooling.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 360
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Scheduler trigger evaluation deterministically decides `defer` vs `trigger` based on configured count/window thresholds.
- AC-2: Adapter can persist one merkle batch row and assign message rows with deterministic `merkle_batch_id`/`merkle_leaf_index` updates.
- AC-3: Adapter can persist `submitted` and `confirmed` merkle batch state transitions with fail-closed validation.
- AC-4: Unit, Functional, Integration, and Regression tests for this scope pass, plus `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | pending messages below thresholds | no trigger decision |
| C-02 | AC-1 | Functional | pending messages at count/window threshold | deterministic trigger decision with stable reason |
| C-03 | AC-2 | Integration | live adapter create-batch + assign-message flow | persisted batch row and message assignment rows updated |
| C-04 | AC-3 | Regression | invalid batch id, negative block height, empty tx hash | fail-closed error variants |
| C-05 | AC-4 | Verification | fmt/clippy + targeted tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m1_batch_scheduler`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test data_layer_postgres_migration_contract`

## Success Metrics
- M1 has deterministic scheduler gating contracts rather than ad-hoc trigger logic.
- PostgreSQL adapter can persist merkle-batch lifecycle state required by story `#5250`.
