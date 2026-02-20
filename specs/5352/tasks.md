# Issue #5352 Tasks

## Ordered Tasks
- [ ] T1 (Conformance Red): add docs-contract test for `#5352` role-pair markers and capture failing run.
- [ ] T2 (Functional/Integration): add daemon role-pair matrix assertions for canonical pair ordering and deterministic reason/taxonomy outcomes.
- [ ] T3 (Documentation): add `#5352` role-pair marker contracts and command references in `docs/ops/configuration.md`.
- [ ] T4 (Conformance Green): run targeted docs-contract + daemon role-pair matrix tests mapped to ACs.
- [ ] T5 (Governance): update R45 next-frontier narrative for role-pair hardening increment.
- [ ] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
