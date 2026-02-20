# Issue #5346 Tasks

## Ordered Tasks
- [x] T1 (Conformance Red): add docs-contract test for `#5346` runtime-to-matrix taxonomy bridge markers and capture failing run.
- [x] T2 (Functional/Integration): add daemon taxonomy-bridge assertions for canonical bridge mapping and repeated-run taxonomy stability.
- [x] T3 (Documentation): add `#5346` bridge marker contracts and command references in `docs/ops/configuration.md`.
- [x] T4 (Conformance Green): run targeted docs-contract + daemon taxonomy bridge tests mapped to ACs.
- [x] T5 (Governance): update R45 next-frontier narrative for taxonomy-bridge hardening increment.
- [x] T6 (Quality): run `cargo fmt --check` and scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 RED evidence.
- T4 depends on T2+T3.
- T6 runs after implementation and conformance checks.
