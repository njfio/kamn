# Issue #3880 Tasks

- Issue: #3880
- Status: Completed

## Ordered Tasks
- T1 (Red): add failing regression tests for invalid-profile reason taxonomy drift.
- T2 (Green): implement and wire stable reason-code assertions for invalid profile cases.
- T3 (Regression): preserve existing production fallback reason taxonomy behavior.
- T4 (Verify): run:
  - cargo test -p kamn-node main_tests::runtime_tests::regression_transport_profile_pair_disallowed_reason_code_is_stable -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::regression_transport_profile_fallback_marker_linkage_reason_code_is_stable -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_live_and_fallback_profile_pair_conflict -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::functional_transport_profile_classifier_rejects_fallback_marker_without_profile_pair -- --exact
  - cargo test -p kamn-node main_tests::runtime_tests::functional_production_transport_profile_classifier_rejects_in_memory_fallback -- --exact

## Completion Evidence
- Invalid-profile transport profile reason taxonomy is now locked by regression tests and remains deterministic.
