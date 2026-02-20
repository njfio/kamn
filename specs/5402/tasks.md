# Issue #5402 Tasks

## Ordered Tasks
- [ ] T1 (Conformance Red): add docs-contract test for `#5402` daemon-test decomposition markers and capture failing run.
- [ ] T2 (Refactor): extract live-postgres fixture/topology/hash constants + helpers into `daemon_tests/live_postgres_fixtures.rs` and wire imports.
- [ ] T3 (Conformance Path Stability): run representative unchanged live-postgres test commands under `main_tests::daemon_tests::...`.
- [ ] T4 (Documentation): add `#5402` decomposition markers in `docs/ops/configuration.md`.
- [ ] T5 (Governance): update R45 next-frontier narrative for daemon-tests decomposition phase-1.
- [ ] T6 (Conformance Green): run docs-contract + daemon representative tests + `wc -l` objective check.
- [ ] T7 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T4 depend on T1 RED evidence.
- T3 depends on T2.
- T6 depends on T2+T3+T4.
- T7 runs after conformance checks.
