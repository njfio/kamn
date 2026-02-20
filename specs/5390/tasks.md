# Issue #5390 Tasks

## Ordered Tasks
- [ ] T1 (Conformance Red): add docs-contract test for `#5390` host/lane composite coherence markers and capture failing run.
- [ ] T2 (Functional/Integration): add daemon host/lane composite coherence assertions and stability checks.
- [ ] T3 (Documentation): add `#5390` coherence marker contracts and command references in `docs/ops/configuration.md`.
- [ ] T4 (Conformance Green): run targeted docs-contract + daemon coherence tests mapped to ACs.
- [ ] T5 (Governance): update R45 next-frontier narrative for host/lane composite coherence hardening increment.
- [ ] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
