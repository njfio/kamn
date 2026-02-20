# Issue #5368 Tasks

## Ordered Tasks
- [ ] T1 (Conformance Red): add docs-contract test for `#5368` host-pair markers and capture failing run.
- [ ] T2 (Functional/Integration): add daemon host-pair assertions and stability/permutation checks.
- [ ] T3 (Documentation): add `#5368` host-pair marker contracts and command references in `docs/ops/configuration.md`.
- [ ] T4 (Conformance Green): run targeted docs-contract + daemon host-pair tests mapped to ACs.
- [ ] T5 (Governance): update R45 next-frontier narrative for host-pair hardening increment.
- [ ] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
