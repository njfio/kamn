# Issue #5370 Tasks

## Ordered Tasks
- [x] T1 (Conformance Red): add docs-contract test for `#5370` directionality markers and capture failing run.
- [x] T2 (Functional/Integration): add daemon directionality assertions and stability checks.
- [x] T3 (Documentation): add `#5370` directionality marker contracts and command references in `docs/ops/configuration.md`.
- [x] T4 (Conformance Green): run targeted docs-contract + daemon directionality tests mapped to ACs.
- [x] T5 (Governance): update R45 next-frontier narrative for directionality hardening increment.
- [x] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
