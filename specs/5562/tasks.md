# Issue #5562 Tasks - PRD Phase-3 kamn-e2e-harness Scaffold and Core Scenario Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for harness required paths, mode/scenario inventory, and manifest/verify contracts.
2. T2 (impl/green): add `crates/kamn-e2e-harness` scaffold and workspace wiring.
3. T3 (impl/green): implement deterministic execution-mode and scenario registries.
4. T4 (impl/green): implement evidence manifest and verifier scaffolds.
5. T5 (docs/green): add phase-3 docs/research status markers and update milestone index active issue set.
6. T6 (verify): run fmt/clippy/targeted tests and capture RED->GREEN evidence.
7. T7 (closeout): set spec status to Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: mode/scenario parser + manifest verifier tests
- Functional: harness CLI mode/scenario plan tests
- Conformance: C-01..C-09
- Integration: driver/scenario/evidence module composition
- Regression: rerun targeted phase-1/phase-2/phase-3 suites
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A (documented in PR)
