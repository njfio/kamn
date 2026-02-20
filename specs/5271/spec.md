# Issue #5271 Spec

- Title: Task: implement M1 finality-observation reconciliation projection
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M1 orchestrator now projects deterministic follow-up policy during submission, but it has no deterministic reconciliation contract for later finality observations from runtime commit finality checks. This leaves a gap between poll-time finality data and confirmation persistence projection required by story `#5250`.

## Scope
In:
- Add a finality-observation input contract for M1 reconciliation.
- Add deterministic reconciliation projection from planned outcomes to:
  - updated follow-up policy,
  - optional confirmation metadata projection.
- Add fail-closed validation for commit-id mismatch and finality=Final without block height.
- Add functional/integration/regression tests for pending/final/failed and fail-closed branches.

Out:
- Daemon polling loop/thread orchestration.
- External transport/network polling implementation changes.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 300
- shell_to_rust_ratio_delta_estimate: -0.0005
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Planned M1 outcomes reconcile finality observations into deterministic follow-up policy updates.
- AC-2: Final observations with valid block height project deterministic confirmation metadata.
- AC-3: Reconciliation fails closed for commit-id mismatch and finality=Final without block-height.
- AC-4: Unit, Functional, Integration, and Regression tests for this slice pass with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | pending finality observation for planned outcome | follow-up policy remains poll-confirmation with deterministic next poll |
| C-02 | AC-2 | Functional | final finality observation with block height | follow-up policy no-retry-final + confirmation metadata projected |
| C-03 | AC-1 | Functional | failed finality observation | follow-up policy no-retry-failed with no confirmation metadata |
| C-04 | AC-3 | Regression | commit-id mismatch or final-finality missing block height | fail-closed error |
| C-05 | AC-4 | Integration | planned outcome + finality observation reconciliation in adapter-lifecycle path | reconciliation and persistence projection remain coherent |
| C-06 | AC-4 | Verification | fmt/clippy + targeted tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test public_api_surface_policy`

## Success Metrics
- Story `#5250` has explicit deterministic contract coverage for post-submission finality reconciliation.
- M1 anchoring runtime contract can bridge finality poll outputs into auditable persistence projection without shell surface growth.

## Verification Evidence
- `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator --test data_layer_postgres_execution_adapter --test public_api_surface_policy` ✅
- `cargo fmt --check` ✅
- `cargo clippy -p kamn-core --tests -- -D warnings` ✅
