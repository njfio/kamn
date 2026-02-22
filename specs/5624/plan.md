# Plan: #5624 Evidence Contract Status Integration

## Approach
1. Compute deterministic evidence contract status/counters in `execute_run_contract`.
2. Emit new top-level `evidence_contract` JSON object.
3. Drive `live_execution.evidence_status` from evidence contract status.
4. Add RED/GREEN tests for evidence object presence, wiring, and failure path.
5. Add docs artifact + docs contract test and update R53 milestone index issue tracking.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/` (R53 evidence-contract artifact)
- `crates/kamn-e2e-harness/tests/` (new docs contract test)
- `specs/milestones/r53-e2e-scenario-execution-activation/index.md`

## Risks and Mitigations
- Risk: output shape change may break strict string assertions.
  - Mitigation: add deterministic field ordering and targeted tests.
- Risk: status wiring mismatch with live validation/orchestration.
  - Mitigation: derive overall status from all component statuses in one place.

## Interfaces / Contracts
- New output object:
  - `evidence_contract:{"expected_artifacts":u64,"recorded_artifacts":u64,"status":"PASS|FAIL"}`
- Updated semantics:
  - `live_execution.evidence_status` now sourced from `evidence_contract.status`.

## ADR
- Not required.
