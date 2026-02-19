# Issue #5107 Tasks

- Issue: #5107
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing tests for requester/sender/recipient field-scoped invalid DID taxonomy in M2.
- [x] T2 (Green): implement canonical KAMN DID parser integration in M2 and remove local duplicated DID validator.
- [x] T3 (Refactor): preserve existing auth/ABAC/negative-matrix/audit semantics while updating DID taxonomy.
- [x] T4 (Regression): run `cargo fmt --check`, `cargo clippy -p kamn-core -- -D warnings`, `cargo test -p kamn-core --test data_layer_m2_gateway_access`, and `cargo test -p kamn-core`.
- [x] T5 (Governance): run shell guardrail evidence commands and confirm `shell_loc_delta_actual = 0`.
- [x] T6 (Verify): mark spec/plan/tasks implemented and close issue with DoD markers.
