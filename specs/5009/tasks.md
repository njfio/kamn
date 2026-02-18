# Issue #5009 Tasks

- Issue: #5009
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M6 red conformance tests for graph registration,
      trust-propagation ranking, and owner-isolation deny paths in child task `#5022`.
- [x] T2 (Green): implement M6 graph integration contracts and exports in child
      task `#5022`.
- [x] T3 (Refactor): tighten trust-ranking and reason-marker behavior in child
      task `#5022`.
- [x] T4 (Regression): run `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m6_graph_integration`, and
      `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for
      child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story
      issue with linked child evidence.
