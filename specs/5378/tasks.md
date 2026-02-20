# Issue #5378 Tasks

## Ordered Tasks
- [ ] T1 (Conformance Red): add docs-contract test for `#5378` host-mode mapping markers and capture failing run.
- [ ] T2 (Functional/Integration): add daemon host-mode mapping assertions and stability checks.
- [ ] T3 (Documentation): add `#5378` host-mode mapping marker contracts and command references in `docs/ops/configuration.md`.
- [ ] T4 (Conformance Green): run targeted docs-contract + daemon host-mode mapping tests mapped to ACs.
- [ ] T5 (Governance): update R45 next-frontier narrative for host-mode mapping hardening increment.
- [ ] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
