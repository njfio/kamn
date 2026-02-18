# Issue #5031 Tasks

- Issue: #5031
- Status: Done

## Ordered Tasks
- [x] T1 (Red / C-01..C-04): add failing conformance tests for ABAC reason
      constants, negative-matrix all-denied/drift-detected outcomes, and
      fail-closed invalid matrix input.
- [x] T2 (Green / AC-1..AC-3): implement additive M2 contracts for
      `DataLayerM2NegativeAuthorizationCase`,
      `DataLayerM2NegativeAuthorizationAuditFixture`,
      `DataLayerM2NegativeAuthorizationMatrixDecision`,
      `DataLayerM2NegativeAuthorizationMatrixReport`, and
      `DataLayerM2AbacEngine::evaluate_negative_authorization_matrix(...)`,
      including stable reason-code constants.
- [x] T3 (Refactor / AC-1): replace M2 ABAC string literals with exported
      constants and re-export new symbols through `kamn-core` crate root.
- [x] T4 (Regression / C-05): run
      `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m2_gateway_access`,
      and `cargo test -p kamn-core`.
- [x] T5 (Governance / C-06): run shell-ratio and shell-ceiling guardrails and
      confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): update lifecycle status markers to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, then open PR and
      close issue with Shell-Surface DoD markers.
