# Issue #3880 Spec

- Title: Subtask: add invalid-profile fail-closed reason taxonomy regression checks
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Invalid profile rejection reasons must remain deterministic for operator debugging and policy gating.

## Scope
In:
- Add reason-code checks for invalid activation paths.

Out:
- Cutover evidence lanes.

## Acceptance Criteria
- AC-1:  Invalid-profile rejections emit stable reason taxonomy.
- AC-2:  Drift triggers deterministic regression failures.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `cargo test -p kamn-node main_tests::runtime_tests::regression_transport_profile_pair_disallowed_reason_code_is_stable -- --exact` | invalid live/fallback profile pair emits stable reason code `runtime_transport_profile_pair_disallowed`. |
| C-02 | AC-1 | Regression | `cargo test -p kamn-node main_tests::runtime_tests::regression_transport_profile_fallback_marker_linkage_reason_code_is_stable -- --exact` | fallback marker linkage violation emits stable reason code `runtime_transport_profile_fallback_marker_without_in_memory_profile`. |
| C-03 | AC-2 | Functional/Regression | `cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_live_and_fallback_profile_pair_conflict -- --exact` | taxonomy drift in invalid-pair classification fails closed. |
| C-04 | AC-2 | Functional/Regression | `cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_fallback_marker_without_profile_pair -- --exact` | taxonomy drift in fallback linkage classification fails closed. |
| C-05 | AC-3 | Regression | `cargo test -p kamn-node main_tests::runtime_tests::functional_production_transport_profile_classifier_rejects_in_memory_fallback -- --exact` | existing production fallback reason taxonomy remains stable. |

## Test Mapping
- `crates/kamn-node/src/main_tests/runtime_tests.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
