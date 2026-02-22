# Spec: #5624 Evidence Contract Status Integration

- Issue: #5624
- Milestone: R53 E2E Scenario Execution Activation
- Status: Reviewed
- Priority: P1

## Problem Statement
`execute_run_contract` currently lacks an explicit evidence contract marker object. `live_execution.evidence_status` is not tied to independently reported evidence readiness semantics, limiting traceability for evidence-phase validation.

## Scope
### In Scope
- Add `evidence_contract` object to run output with deterministic fields.
- Wire `live_execution.evidence_status` to `evidence_contract.status`.
- Support deterministic evidence failure marker path for contract validation.

### Out of Scope
- Real filesystem evidence emission changes.
- Runtime orchestration/process management changes.
- New dependencies.

## Acceptance Criteria
### AC-1 Evidence contract object
Given run output generation,
When `execute_run_contract` returns JSON,
Then top-level output contains `evidence_contract` with stable fields and deterministic values.

### AC-2 Evidence status wiring
Given `evidence_contract.status`,
When `live_execution` is rendered,
Then `live_execution.evidence_status == evidence_contract.status`.

### AC-3 Deterministic failure path
Given evidence fail marker path,
When run output is rendered,
Then `evidence_contract.status=FAIL` and `live_execution.overall_status=FAIL`.

### AC-4 Runtime marker stability
Given prior runtime marker contracts,
When this change is applied,
Then `runtime_*`, `spawn_*`, and `process_*` marker structures remain unchanged.

## Conformance Cases
- C-01 (AC-1, Functional/Conformance): run output contains `evidence_contract` with expected fields.
- C-02 (AC-2, Functional/Conformance): evidence status mirrors into `live_execution.evidence_status`.
- C-03 (AC-3, Regression/Conformance): `/tmp/evidence-fail` path sets evidence status FAIL and overall FAIL.
- C-04 (AC-4, Regression): runtime marker objects remain unchanged.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new evidence-contract assertions.
- `cargo test -p kamn-e2e-harness` green.
