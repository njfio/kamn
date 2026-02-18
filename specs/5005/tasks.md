# Issue #5005 Tasks

- Issue: #5005
- Status: Done

## Ordered Tasks
- [x] T1 (Red): define M2 red conformance tests for DID auth/session issuance,
      ABAC allow/deny matrix, negative matrix drift detection, RLS policy
      contract markers, and access-audit hash-chain behavior in child task
      `#5018`.
- [x] T2 (Green): implement M2 gateway contracts and exports in child task
      `#5018`.
- [x] T3 (Refactor): tighten deterministic reason markers and fail-closed error
      semantics in child task `#5018`.
- [x] T4 (Regression): run `cargo fmt --check`,
      `cargo clippy -p kamn-core -- -D warnings`,
      `cargo test -p kamn-core --test data_layer_m2_gateway_access`, and
      `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero for
      child delivery (`shell_loc_delta_actual = 0`).
- [x] T6 (Verify): set story lifecycle artifacts to
      `spec=Implemented`, `plan=Implemented`, `tasks=Done`, and close story
      issue with linked child evidence.
