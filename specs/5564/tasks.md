# Issue #5564 Tasks - PRD Phase-4a Scenario Matrix and Evidence Verifier Contract Completion

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for full scenario matrix inventory (`S-01..S-15`), run-plan scenario count, manifest field contracts, and deterministic verifier report markers.
2. T2 (impl/green): add missing scenario modules and extend scenario registry contracts to full PRD matrix.
3. T3 (impl/green): update harness run-plan builder to schedule full scenario matrix.
4. T4 (impl/green): extend evidence manifest model with PRD section-8.2 infrastructure/scenario/summary markers.
5. T5 (impl/green): implement deterministic verification report structure and renderer with schema/proof/chain/content checks.
6. T6 (docs/green): add phase-4a research artifact markers and update milestone index active/completed issue set.
7. T7 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness` and targeted regression suites.
8. T8 (closeout): set spec status to Implemented and complete issue/PR lifecycle updates.

## Test Tier Mapping
- Unit: scenario registry mapping, manifest model constructors, verifier report renderer.
- Functional: `build_core_run_plan` scheduling behavior and `verify_manifest`/report checks.
- Conformance: C-01..C-13.
- Integration: scenario + manifest + verifier composition from crate public API.
- Regression: rerun phase-1/phase-2/phase-3/phase-4a targeted suites for regressions.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for this contract/model slice (documented in PR).
