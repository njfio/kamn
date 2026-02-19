# Issue #5091 Tasks

- Issue: #5091
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing tests for malformed agent DID rejection on auth,
  scope validation, and agent-role authorization paths.
- [x] T2 (Green): integrate `AgentDid::parse` into M2 validation paths for
  requester/sender/recipient agent DID checks.
- [x] T3 (Refactor): keep non-agent role validation behavior stable while
  removing duplicate agent-format logic in affected code paths.
- [x] T4 (Regression): run `cargo fmt --check`,
  `cargo clippy -p kamn-core -- -D warnings`,
  `cargo test -p kamn-core --test data_layer_m2_gateway_access`, and
  `cargo test -p kamn-core`.
- [x] T5 (Governance): confirm shell/workflow/python/template delta is zero and
  record DoD shell markers.
- [x] T6 (Verify): set lifecycle markers to Implemented/Done and post issue
  closure evidence.
