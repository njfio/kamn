# Issue #5011 Tasks

- Issue: #5011
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M8 red conformance tests for retention windows, legal-hold precedence,
      crypto-shredding integrity markers, and owner-scope deny paths in child task `#5024`.
- [x] T2 (Green): implement M8 compliance contracts and exports in child task `#5024`.
- [x] T3 (Refactor): tighten deterministic reason-marker behavior and fail-closed taxonomy in child task `#5024`.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m8_compliance_lifecycle`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story issue with linked child evidence.
