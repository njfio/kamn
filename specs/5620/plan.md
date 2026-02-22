# Plan: #5620 SCENARIO_RUN Driver Execution Contract

## Approach
1. Introduce deterministic scenario execution helper in `crates/kamn-e2e-harness/src/lib.rs` that:
   - resolves driver from `ExecutionMode` (`sdk_direct`, `cli_scripted`, `mcp_agent`),
   - executes each selected scenario once,
   - returns normalized status markers (`PASS`/`FAIL`/`SKIP`) and ordered results.
2. Extend run-output JSON contract with `scenario_results` array.
3. Update phase computation so `SCENARIO_RUN` step/status/details derive from scenario result statuses.
4. Preserve existing runtime marker blocks unchanged to satisfy prior contract tests.
5. Add conformance tests in `crates/kamn-e2e-harness/tests/command_contract.rs` and a docs contract test for R53 traceability.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/` (new R53 docs contract test)
- `docs/research/` (R53 gap-analysis artifact)
- `specs/milestones/r53-e2e-scenario-execution-activation/index.md`

## Risks and Mitigations
- Risk: broad changes to heavily-tested run-output JSON shape.
  - Mitigation: add only one new top-level field (`scenario_results`) and keep existing fields stable.
- Risk: existing phase status logic expects scenario/evidence/teardown to skip.
  - Mitigation: scope behavior change to `SCENARIO_RUN` only; keep EVIDENCE/TEARDOWN untouched.
- Risk: inconsistent driver status casing.
  - Mitigation: normalize driver outputs to canonical uppercase status labels before aggregation.

## Interfaces / Contracts
- New run JSON field:
  - `scenario_results: [{"id":"S-01","status":"PASS"}, ...]`
- `SCENARIO_RUN` phase step detail changes from placeholder skip to deterministic aggregate of scenario execution.

## ADR
- Not required: no dependency, protocol, or architecture boundary change.
