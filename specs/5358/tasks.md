# Issue #5358 Tasks

## Ordered Tasks
- [x] T1 (Conformance Red): add docs-contract test for `#5358` order-invariance markers and capture failing run.
- [x] T2 (Functional/Integration): add daemon order-invariance assertions for canonical fingerprinting and baseline/permuted lane-order equivalence.
- [x] T3 (Documentation): add `#5358` order-invariance marker contracts and command references in `docs/ops/configuration.md`.
- [x] T4 (Conformance Green): run targeted docs-contract + daemon order-invariance tests mapped to ACs.
- [x] T5 (Governance): update R45 next-frontier narrative for order-invariance hardening increment.
- [x] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
