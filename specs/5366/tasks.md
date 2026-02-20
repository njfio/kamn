# Issue #5366 Tasks

## Ordered Tasks
- [x] T1 (Conformance Red): add docs-contract test for `#5366` topology permutation markers and capture failing run.
- [x] T2 (Functional/Integration): add daemon topology permutation assertions and invariance checks.
- [x] T3 (Documentation): add `#5366` topology permutation marker contracts and command references in `docs/ops/configuration.md`.
- [x] T4 (Conformance Green): run targeted docs-contract + daemon topology permutation tests mapped to ACs.
- [x] T5 (Governance): update R45 next-frontier narrative for topology permutation hardening increment.
- [x] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
