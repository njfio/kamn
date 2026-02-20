# Issue #5340 Tasks

## Ordered Tasks
- [x] T1 (Conformance Red): add docs-contract test for `#5340` gate/deferred markers and capture failing run.
- [x] T2 (Regression/Unit): add deterministic env-unset and env-precedence tests for live-postgres gate decision helper.
- [x] T3 (Integration): add env-gated deferred-path daemon live-postgres validation test.
- [x] T4 (Documentation): add gate/deferred marker contracts and command references in `docs/ops/configuration.md`.
- [x] T5 (Conformance Green): run targeted docs-contract and daemon test commands mapped to ACs.
- [x] T6 (Governance): update R45 review next-frontier narrative for this hardening increment.
- [x] T7 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3/T4 depend on T1 RED evidence.
- T5 depends on T2+T3+T4.
- T7 runs after implementation and conformance checks.
