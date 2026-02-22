# Issue #5576 Tasks - PRD Phase-4g Lifecycle Summary Aggregation Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for lifecycle summary presence and deterministic totals.
2. T2 (impl/green): add summary aggregation helper structs/functions for phase and step totals.
3. T3 (impl/green): include `lifecycle_summary` in run output.
4. T4 (docs/green): add phase-4g docs marker artifact and milestone index update.
5. T5 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, and phase-1/2 regressions.
6. T6 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: summary aggregation helper tests.
- Functional: run output summary behavior (normal + fail-path).
- Conformance: C-01..C-09.
- Integration: parser + mode + phase/result/step + summary composition.
- Regression: phase-1/2/4 suite reruns.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic contract slice.
