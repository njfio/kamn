# Issue #5030 Tasks

- Issue: #5030
- Status: Done

## Ordered Tasks
- [x] T1 (Red / C-01..C-04): add failing conformance tests for proof-decision
      reason constants, anchoring failure-matrix stable/drift outcomes, and
      fail-closed invalid matrix inputs.
- [x] T2 (Green / AC-1..AC-3): implement additive M1 proof-decision and
      anchoring failure-matrix contracts with stable reason constants and typed
      invalid-input errors.
- [x] T3 (Refactor / AC-1..AC-2): keep existing M1 behavior stable while
      exporting new symbols via `kamn-core` crate root.
- [x] T4 (Regression / C-05): run
      `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m1_merkle_anchoring`,
      and `cargo test -p kamn-core`.
- [x] T5 (Governance / C-06): run shell-ratio and shell-ceiling guardrails and
      confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): set lifecycle statuses to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, then open PR and
      close issue with shell-surface DoD markers.
