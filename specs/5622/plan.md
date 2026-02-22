# Plan: #5622 Live Status Alignment with Scenario Outcomes

## Approach
1. Extend run aggregation in `execute_run_contract` to derive live status from scenario execution totals.
2. Keep top-level `live_execution` object shape stable while making `overall_status` data-driven.
3. Keep `live_validation.expected_checks` stable and compute `completed_checks` deterministically from scenario pass/fail outcomes.
4. Add command-contract tests for pass/fail alignment and check accounting.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/` (R53 slice artifact)
- `crates/kamn-e2e-harness/tests/` (new docs contract test)
- `specs/milestones/r53-e2e-scenario-execution-activation/index.md`

## Risks and Mitigations
- Risk: changing live status semantics could break existing tests.
  - Mitigation: add RED tests first and preserve field names.
- Risk: inconsistency between scenario and phase status.
  - Mitigation: compute from same scenario totals used in SCENARIO_RUN phase.

## Interfaces / Contracts
- Stable fields:
  - `live_execution.{orchestration_status,validation_status,evidence_status,overall_status}`
  - `live_validation.{expected_checks,completed_checks,status}`
- Changed semantics:
  - `overall_status` and `status` now reflect scenario outcomes.

## ADR
- Not required.
