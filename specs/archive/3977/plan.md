# Plan — Issue #3977

## Approach

1. Add runtime-budget status computation and deterministic runtime-budget reason code in `check_kamn_core_rustdoc_artifact_policy.sh`.
2. Update `test_check_kamn_core_rustdoc_artifact_policy.sh` with:
   - pass assertion for `runtime_budget_status=within`
   - runtime-budget fail fixture asserting deterministic reason code.
3. Update `test_run_kamn_core_rustdoc_artifact_contract_lane.sh` to assert runtime budget status marker from policy checker.
4. Align docs marker strings in `docs/ci/strategy.md` and `docs/architecture/runtime.md`, and update `scripts/ci/test_ci_strategy_contract.sh` marker expectations.
5. Run targeted rustdoc-policy/docs-contract tests and fast CI tools regression.

## Affected Paths

- `scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh`
- `scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`
- `scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`
- `scripts/ci/test_ci_strategy_contract.sh`
- `docs/ci/strategy.md`
- `docs/architecture/runtime.md`
- `specs/3977/spec.md`
- `specs/3977/plan.md`
- `specs/3977/tasks.md`

## Risks / Mitigations

- Risk: Expanding reason-code CSV surface can drift docs/tests.
  Mitigation: update checker + docs + CI strategy contract in same change.

- Risk: Runtime-budget fail contract could be flaky if tied to real runtime.
  Mitigation: use crafted JSON fixture with explicit `runtime_seconds > max_runtime_seconds` in policy checker tests.

## ADR

- Not required (contract/test/documentation governance update only).
