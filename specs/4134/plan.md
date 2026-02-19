# Issue #4134 Plan

- Issue: #4134
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Approach
1. Confirm runtime contract-lane checker and drift-fail paths for invariant/fuzz/concurrency evidence.
2. Confirm CI selector routing keeps local-heavy paths explicit opt-in/local-only by default.
3. Confirm docs-contract assertions preserve boundary/taxonomy markers.
4. Record closure evidence and mark task/subtasks implemented.

## Affected Files
- `scripts/runtime/check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`
- `scripts/runtime/invariant_fuzz_concurrency_contract_lane_contract.sh`
- `scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `scripts/ci/select_targets.sh`
- `scripts/ci/test_select_targets.sh`
- `docs/ci/strategy.md`
- `specs/4134/spec.md`
- `specs/4134/plan.md`
- `specs/4134/tasks.md`
- `specs/4141/{spec.md,plan.md,tasks.md}`
- `specs/4142/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: Drift between checker schema fields and docs marker contracts.
  - Mitigation: keep coverage in both shell contract tests and Rust docs-contract tests.
- Risk: Local-heavy selectors accidentally enabled in fast-gate defaults.
  - Mitigation: enforce selector behavior with `scripts/ci/test_select_targets.sh`.
- Risk: governance-only closure increases shell surface.
  - Mitigation: no new shell scripts/wrappers; closure uses existing test infrastructure.

## Interfaces / Contracts
- Policy checker contract: deterministic reason taxonomy + fail-closed exit on drift.
- CI selector contract: local-heavy concurrency paths remain explicit opt-in/local-only.
- Docs contract: concurrency boundary markers remain stable and asserted.
