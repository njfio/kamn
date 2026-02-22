# Spec: #5620 SCENARIO_RUN Driver Execution Contract

- Issue: #5620
- Milestone: R53 E2E Scenario Execution Activation
- Status: Reviewed
- Priority: P1
- Owner: codex

## Problem Statement
`kamn-e2e-harness` currently emits placeholder `SCENARIO_RUN` phase behavior (`SKIP`) and does not execute selected scenarios through mode drivers during `execute_run_contract`. This diverges from PRD section 11.2 phase-3 expectations and obscures run-result semantics for scenario execution.

## Scope
### In Scope
- Execute selected scenarios once each via mode-specific driver in `execute_run_contract`.
- Add deterministic `scenario_results` output entries preserving selected scenario order.
- Set `SCENARIO_RUN` phase status/details/step status from scenario execution outcomes.
- Preserve runtime external execution + preflight + orchestration marker contracts.

### Out of Scope
- Real infrastructure process orchestration changes.
- Evidence file I/O changes beyond run-output JSON contract.
- CI workflow/template updates.

## Acceptance Criteria
### AC-1 Driver Execution
Given a valid run config and selected scenario IDs,
When `execute_run_contract` is called,
Then each selected scenario is executed exactly once by the driver bound to `mode`.

### AC-2 Scenario Result Output
Given scenario execution completed,
When run output JSON is returned,
Then output contains `scenario_results` array with entries `{id,status}` for each selected scenario in input order.

### AC-3 SCENARIO_RUN Pass Semantics
Given all executed scenarios report `PASS`,
When run output phase results are computed,
Then `SCENARIO_RUN` phase status is `PASS` and is not `SKIP`.

### AC-4 SCENARIO_RUN Fail Semantics
Given one or more executed scenarios report `FAIL`,
When run output phase results are computed,
Then `SCENARIO_RUN` phase status is `FAIL` and lifecycle summary fail counts increase accordingly.

### AC-5 Contract Stability
Given existing external-execution contracts,
When run output is produced after this change,
Then existing runtime preflight/orchestration/lifecycle/validation markers remain present and semantically unchanged.

## Conformance Cases
- C-01 (AC-1, Unit/Functional): `sdk-direct` selected scenarios execute once each; output count equals input count.
- C-02 (AC-1, Unit/Functional): `mcp-tau` mode selects MCP driver path and executes each selected scenario once.
- C-03 (AC-2, Conformance): output includes `scenario_results` array preserving `--scenarios` input ordering.
- C-04 (AC-3, Conformance): all-pass scenario execution sets `SCENARIO_RUN` status to `PASS`.
- C-05 (AC-4, Regression/Conformance): fail-path marker forces at least one scenario to `FAIL`; `SCENARIO_RUN` and lifecycle summary both reflect failure.
- C-06 (AC-5, Regression): runtime execution marker objects remain present with prior field names and values for enabled/disabled external execution.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` passes with new scenario execution assertions.
- `cargo test -p kamn-e2e-harness` remains green.
- No regressions in existing R51/R52 contract tests.
