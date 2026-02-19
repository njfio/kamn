# Issue #4141 Plan

- Issue: #4141
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Approach
1. Validate checker fail-closed behavior for drifted marker lineage/taxonomy fields.
2. Validate selector routing for deterministic local-heavy exclusion behavior.
3. Validate bounded runtime CI-smoke contract-lane execution.
4. Record AC-to-test evidence and mark subtask implemented.

## Affected Files
- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `scripts/ci/select_targets.sh`
- `scripts/ci/test_select_targets.sh`
- `specs/4141/spec.md`
- `specs/4141/plan.md`
- `specs/4141/tasks.md`

## Risks and Mitigations
- Risk: taxonomy marker drift can silently weaken governance checks.
  - Mitigation: keep deterministic reason-code contract assertions and fail-closed exits.
- Risk: selector drift could leak local-heavy paths into fast-gate.
  - Mitigation: maintain explicit local-heavy scope assertions in selector matrix tests.

## Interface Contract
- Policy checker outputs deterministic marker keys and reason code values.
- Selector outputs deterministic lane scope markers for local-heavy boundaries.
