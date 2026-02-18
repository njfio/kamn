# Issue #5007 Tasks

- Issue: #5007
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M4 red conformance tests for lifecycle transitions,
      escrow visibility matrix, and settlement evidence tamper detection in child
      task `#5020`.
- [x] T2 (Green): implement M4 escrow integration contracts and exports in child
      task `#5020`.
- [x] T3 (Refactor): tighten transition/authorization/error-marker behavior in
      child task `#5020`.
- [x] T4 (Regression): run `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m4_escrow_integration`, and
      `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for
      child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story
      issue with linked child evidence.
