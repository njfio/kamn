# Issue 6258 Tasks

- [x] T1 (Red): inventory and assert current root-public module surface.
- [x] T2 (Green): tighten `kamn-core` module visibility and remove shim root modules.
- [x] T3 (Green): migrate in-workspace module-path callsites to curated `kamn_core` exports.
- [x] T4 (Regression): refresh API-surface policy baseline fixture.
- [x] T5 (Verification): run conformance checks C-01..C-03.
- [x] T6 (Verification): run targeted suites:
  - `cargo test -p kamn-core --test public_api_surface_policy`
  - `cargo test -p kamn-types`
  - `cargo test -p kamn-node`
