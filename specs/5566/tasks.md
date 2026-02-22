# Issue #5566 Tasks - PRD Phase-4b Harness Run/Verify Command Contracts

## Ordered Tasks
1. T1 (tests/red): add failing conformance tests for command parser, scenario CSV selection/validation, and verify output-file/report marker contracts.
2. T2 (impl/green): add command parser data types/helpers in harness library and wire `run`/`verify` command dispatch in binary.
3. T3 (impl/green): implement scenario selection parser against `S-01..S-15` matrix.
4. T4 (impl/green): implement verify output writing with deterministic report generation.
5. T5 (docs/green): add phase-4b docs/research markers and milestone index state update.
6. T6 (verify): run `cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, `cargo test -p kamn-e2e-harness`, plus phase-1/2 regressions.
7. T7 (closeout): update spec status to Implemented and complete issue/PR lifecycle artifacts.

## Test Tier Mapping
- Unit: command/scenario parser helpers.
- Functional: run/verify command execution outputs.
- Conformance: C-01..C-11.
- Integration: parser + scenario registry + verify output composition.
- Regression: phase-1/2/4a targeted suites.
- Property/Contract/Snapshot/Fuzz/Mutation/Performance: N/A for command-contract slice (documented in PR).
