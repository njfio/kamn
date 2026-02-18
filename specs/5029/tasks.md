# Issue #5029 Tasks

- Issue: #5029
- Status: Done

## Ordered Tasks
- [x] T1 (Red / C-01..C-03): add failing conformance tests for stable/drift
      matrix decisions and invalid matrix input fail-closed behavior.
- [x] T2 (Green / AC-1..AC-3): implement additive M0 conformance-matrix
      contracts/API, stable decision reason constants, and typed invalid-input
      errors.
- [x] T3 (Refactor / AC-1..AC-2): keep existing M0 record/ledger behavior
      unchanged while re-exporting new matrix symbols via `kamn-core`.
- [x] T4 (Regression / C-04): run
      `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m0_contract`,
      and `cargo test -p kamn-core`.
- [x] T5 (Governance / C-05): run shell-ratio and shell-ceiling guardrails and
      confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): set lifecycle statuses to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, then open PR and
      close issue with shell-surface DoD markers.
