# Issue #5572 Tasks - PRD Phase-4e Orchestration Lifecycle Step-Record Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for nested `steps` fields and required INFRA_UP/AGENT_DEPLOY step markers.
2. T2 (impl/green): add step-record model and integrate nested step rendering in run output.
3. T3 (impl/green): populate deterministic INFRA_UP/AGENT_DEPLOY step lists aligned to PRD section 11.2.
4. T4 (docs/green): add phase-4e docs markers and milestone index progression update.
5. T5 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, and phase-1/2 regressions.
6. T6 (closeout): set spec status Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: step-record model/status contracts.
- Functional: run output step marker coverage.
- Conformance: C-01..C-09.
- Integration: run parser + phase/result/step composition.
- Regression: phase-1/2/4 suite reruns.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for deterministic contract slice.
