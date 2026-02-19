# Issue #5004 Tasks

- Issue: #5004
- Status: Done

## Ordered Tasks
- [x] T1 (Red): complete M1 red tests and conformance scaffolding in child issue
      `#5017`.
- [x] T2 (Green): deliver core M1 trust-anchor contracts in child issue `#5017`.
- [x] T3 (Refactor): add deterministic proof decision and anchoring
      failure-matrix contracts while preserving existing M1 behavior in child
      issue `#5030`.
- [x] T4 (Regression): validate story behavior through
      `cargo test -p kamn-core --test data_layer_m1_merkle_anchoring` and
      `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell guardrails in-go with aggregate
      `shell_loc_delta_actual = 0` across `#5017` and `#5030`.
- [x] T6 (Verify): set story lifecycle artifacts to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story
      issue with linked child evidence.
