# Issue #5338 Tasks

## Ordered Tasks
- [x] T1 (Conformance Red): add docs-contract test for issue `#5338` marker section and capture failing run before docs updates.
- [x] T2 (Integration): add env-gated daemon/live-postgres validation slice test in `daemon_tests`.
- [x] T3 (Documentation): add `#5338` contract marker section in `docs/ops/configuration.md`.
- [x] T4 (Conformance Green): run targeted docs-contract + daemon slice tests (and one live adapter command reference) to verify AC mappings.
- [x] T5 (Governance): update `docs/review/gaps-and-issues-r45.md` next-milestone narrative to reflect initiated tracked slice.
- [x] T6 (Quality): run `cargo fmt --check` plus scoped clippy on touched crates.

## Dependency Notes
- T2/T3 depend on T1 red evidence.
- T4 depends on T2 + T3.
- T6 runs after implementation and conformance checks.
