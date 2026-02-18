# Issue #3879 Spec

- Title: Subtask: add native-fallback profile compatibility validation checks
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r26-4-native-libp2p-production-activation-and-live-node-validation/index.md

## Problem Statement
Compatibility rules must be explicit and enforced to avoid ambiguous startup behavior.

## Scope
In:
- Add compatibility checks for supported profile pairs.

Out:
- Rollback workflow implementation.

## Acceptance Criteria
- AC-1:  Unsupported profile pairs fail closed deterministically.
- AC-2:  Supported pairs pass with stable markers.
- AC-3:  Unit, Functional, Integration, and Regression tests are present and passing (or justified N/A).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit/Functional | `cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_live_and_fallback_profile_pair_conflict -- --exact` | mixed live/fallback profile family fails closed with `runtime_transport_profile_pair_disallowed`. |
| C-02 | AC-1 | Unit/Functional | `cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_fallback_marker_without_profile_pair -- --exact` | fallback marker without in-memory profile fails closed with deterministic reason code. |
| C-03 | AC-2 | Unit/Functional | `cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_accepts_planning_in_memory_profile_pair -- --exact` | supported planning-mode in-memory profile/fallback pair passes. |
| C-04 | AC-3 | Regression | `cargo test -p kamn-node main_tests::runtime_tests::functional_production_transport_profile_classifier_rejects_in_memory_fallback -- --exact` | existing production fallback rejection behavior remains stable. |
| C-05 | AC-3 | Docs/Contract | `cargo test -p kamn-core --test runtime_network_docs` | runtime-network docs contain new transport profile pair reason taxonomy markers. |

## Test Mapping
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `docs/foundation/runtime-network.md`
- `docs/architecture/p2p-transport.md`
- `crates/kamn-core/tests/runtime_network_docs.rs`

## Success Metrics
- All ACs have matching conformance tests and pass in CI/local-heavy lanes as applicable.
