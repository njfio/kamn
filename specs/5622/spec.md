# Spec: #5622 Live Status Alignment with Scenario Outcomes

- Issue: #5622
- Milestone: R53 E2E Scenario Execution Activation
- Status: Reviewed
- Priority: P1

## Problem Statement
`execute_run_contract` currently emits static `live_execution` and `live_validation` PASS markers even when scenario execution can now fail (from #5620). This creates a contract mismatch between scenario results and top-level live status summaries.

## Scope
### In Scope
- Derive `live_execution.overall_status` from scenario/phase outcomes.
- Derive `live_validation.status` and `completed_checks` from deterministic scenario outcome aggregation.
- Preserve existing field names and runtime marker structures.

### Out of Scope
- Runtime orchestration/process spawn behavior changes.
- New dependencies.
- Evidence file format changes outside run output markers.

## Acceptance Criteria
### AC-1 Pass alignment
Given all selected scenarios pass,
When run output is produced,
Then `live_execution.overall_status` is `PASS` and `live_validation.status` is `PASS`.

### AC-2 Fail alignment
Given at least one selected scenario fails,
When run output is produced,
Then `live_execution.overall_status` is `FAIL` and `live_validation.status` is `FAIL`.

### AC-3 Validation completion accounting
Given deterministic check accounting,
When run output is produced,
Then `live_validation.completed_checks` reflects completed checks without over-reporting on failure paths.

### AC-4 Contract stability
Given existing runtime contract fields,
When this change is applied,
Then `runtime_*`, `spawn_*`, and `process_*` markers remain present and semantically unchanged.

## Conformance Cases
- C-01 (AC-1, Functional/Conformance): all-pass scenario execution yields PASS for live_execution + live_validation.
- C-02 (AC-2, Functional/Conformance): scenario fail path yields FAIL for live_execution + live_validation.
- C-03 (AC-3, Regression/Conformance): completed_checks equals expected_checks on pass and is reduced on fail path.
- C-04 (AC-4, Regression): runtime_orchestration/lifecycle/validation marker structures remain unchanged.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new live status assertions.
- `cargo test -p kamn-e2e-harness` green with no regressions.
