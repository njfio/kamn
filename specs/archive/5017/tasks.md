# Issue #5017 Tasks

- Issue: #5017
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add `spec_c01`..`spec_c05` tests for deterministic merkle roots, proof verification, tamper fail-closed behavior, anchoring idempotency, and invalid-input rejection.
- [x] T2 (Green): implement `data_layer_m1` batch/proof/anchoring contracts to satisfy all red tests.
- [x] T3 (Refactor): normalize helper APIs (`leaf_digest`, `node_digest`, proof-step modeling) and tighten error messages/reason codes.
- [x] T4 (Regression): run `cargo test -p kamn-core --test data_layer_m1_merkle_anchoring` and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm zero shell/python/workflow/template changes; record `shell_loc_delta_actual=0`.
- [x] T6 (Verify): set `spec.md` status to `Implemented`, close AC mapping, and update issue process log with commands/results.
