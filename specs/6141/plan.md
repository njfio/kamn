# Plan: Issue #6141

## Approach
1. Capture RED evidence that the fast-gate workflow does not include explicit coverage-guided fuzz contract lane execution/report wiring.
2. Add a dedicated fast-gate step to run:
   - `bash scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh --output-json runtime-input-mutation-coverage-guided-contract-report.json`
3. Add an artifact upload step for the lane report.
4. Extend workflow scope-policy regression tests to assert this lane/report wiring and preserve deep-lane exclusion guarantees.
5. Run scoped verification (`test_workflow_scope_policy`, shell syntax checks) and capture GREEN evidence.

## Affected Modules
- `.github/workflows/ci-fast-gate.yml`
- `scripts/ci/test_workflow_scope_policy.sh`
- `specs/6141/spec.md`
- `specs/6141/plan.md`
- `specs/6141/tasks.md`

## Risks / Mitigations
- Risk: Fast-gate runtime budget regression.
  Mitigation: Use bounded coverage-guided contract lane (existing max-seconds guard).
- Risk: Accidental enablement of deep fuzz lane in CI.
  Mitigation: Keep existing deep-lane exclusion assertions and validate in scope-policy test.
- Risk: Workflow-policy drift.
  Mitigation: Add explicit regression assertions in `test_workflow_scope_policy.sh`.

## Interfaces / Contracts
- No production API/wire contract changes.
- CI workflow contract changes only; must include regression policy test updates in the same patch.
