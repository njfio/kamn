# Issue #5016 Tasks

- Issue: #5016
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add `spec_c01`..`spec_c04` tests for deterministic hashes, duplicate rejection, chain tamper detection, compression metadata validation.
- [x] T2 (Green): implement `data_layer_m0` record + ledger APIs and error taxonomy to satisfy tests.
- [x] T3 (Refactor): finalize canonical serialization helpers and compact hashing utilities with clear docs.
- [x] T4 (Regression): run `cargo test -p kamn-core data_layer_m0` and then `cargo test -p kamn-core`.
- [x] T5 (Governance): report measured shell/rust deltas (expected shell delta 0).
- [x] T6 (Verify): update AC/test matrix evidence and set `spec.md` status to `Implemented`.
