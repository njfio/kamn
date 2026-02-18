# Issue #3879 Tasks

- Issue: #3879
- Status: Completed

## Ordered Tasks
- T1 (Red): add failing runtime profile-pair tests for mixed live/fallback profiles and invalid fallback marker linkage.
- T2 (Green): implement deterministic compatibility classification checks in runtime orchestration.
- T3 (Refactor): keep existing production-only policy checks stable while adding cross-profile pair validation.
- T4 (Regression): preserve existing in-memory fallback rejection for production modes.
- T5 (Docs): update runtime-network and p2p-transport docs with new compatibility reason markers.
- T6 (Verify): run:
  - cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_live_and_fallback_profile_pair_conflict -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_fallback_marker_without_profile_pair -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_accepts_planning_in_memory_profile_pair -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::functional_production_transport_profile_classifier_rejects_in_memory_fallback -- --exact
  - cargo test -p kamn-core --test runtime_network_docs

## Completion Evidence
- Runtime transport profile compatibility checks now fail closed for unsupported live/fallback pairings and fallback-marker linkage drift.
- Scoped runtime tests and runtime-network docs contract suite pass.
