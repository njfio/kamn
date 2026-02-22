# Plan: #5682 Scenario Contract Projection in E2E Harness Run Output

## Approach
1. Add RED tests for scenario contract completeness and PRD-aligned P0 content.
2. Extend `ScenarioDefinition` in `crates/kamn-e2e-harness/src/scenarios/mod.rs` with contract fields.
3. Update scenario modules with deterministic contract metadata (full P0 specificity, non-empty defaults for P1/P2).
4. Add `scenario_contracts` projection to run JSON output in `execute_run_contract` preserving selected ordering.
5. Update/extend command contract tests for projection shape and order.
6. Update phase-3 research markers and run regression gates.

## Affected Modules
- `crates/kamn-e2e-harness/src/scenarios/mod.rs`
- `crates/kamn-e2e-harness/src/scenarios/*.rs`
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/mode_scenario_manifest_contract.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/e2e-live-testing-prd-phase3-gap-analysis.md`

## Risks and Mitigations
- Risk: large string literals increase maintenance burden.
- Mitigation: keep deterministic, concise entries and assert only key PRD-aligned anchors in tests.

- Risk: JSON output growth may break brittle string-match tests.
- Mitigation: add targeted expected-fragment assertions and preserve existing fields.

## Interfaces / Contracts
- `ScenarioDefinition` extends with:
  - `steps: Vec<&'static str>`
  - `verifiable_outputs: Vec<&'static str>`
  - `pass_criteria: Vec<&'static str>`
- `execute_run_contract` output extends with:
  - `scenario_contracts: [{id,name,priority,steps,verifiable_outputs,pass_criteria,status}]`

## ADR
- Not required. No dependency/protocol/wire-format boundary change.
