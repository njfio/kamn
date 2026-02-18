# Plan — Issue #4825

## Approach

- Keep lane entrypoint wrapper files stable and executable while replacing duplicated implementation bodies with `exec` delegation to family-shared impls.
- Introduce a shared harness helper library for deterministic precondition/assertion utilities and migrate representative high-duplication tests first.
- Validate in strict order:
  1. RED: harness migration contract test fails before harness exists.
  2. GREEN: harness and shared impls added; migrated cohorts pass.
  3. Regression: run full `test_ci_tools.sh`.
- Refresh the non-Kolme trend soft-budget baseline fixture after migration-driven LOC reduction so drift checks remain strict.

## Affected Modules

- `scripts/lib/test_harness.sh`
- `scripts/lib/test_test_harness_migration_contract.sh`
- `scripts/ci/test_wave_wrapper_family_baseline_contract_impl.sh`
- `scripts/ci/test_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/test_non_kolme_wave_wrapper_family_baseline_contract_impl.sh`
- `scripts/ci/test_kolme_wave_wrapper_family_baseline_contract_impl.sh`
- `scripts/ci/test_check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/test_check_kolme_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `fixtures/ci/non_kolme_wave_trend_test_loc_soft_budget_baseline.json`

## Risks / Mitigations

- Risk: behavior drift while deduplicating shell logic.
  Mitigation: keep thin wrappers and preserve CLI contract (`--wave-id`) through shared impl delegation; validate entire migrated cohort.
- Risk: false-green soft-budget checks from stale baseline fixtures.
  Mitigation: refresh baseline fixture and keep strict `delta=0` expectation in checker tests.
- Risk: broad CI regressions from shell entrypoint changes.
  Mitigation: run full `bash scripts/ci/test_ci_tools.sh` before PR.

## Interfaces / Contracts

- Wrapper entrypoints remain executable and continue accepting the same `--wave-id` argument shape.
- Shared impls add `--family <kolme|non_kolme>` internally; family wrappers supply it.
- Existing key=value checker output contract is unchanged.

## ADR

- Not required; no new dependency, protocol, or architecture decision.
