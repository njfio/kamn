# Plan — Issue #4824

## Approach

1. Add a RED topology contract test requiring:
   - shared non-Kolme wave budget trend checker runner
   - wave1-wave19 checker symlink entrypoints targeting that runner
2. Implement shared parameterized checker runner that:
   - detects wave id from symlink entrypoint name or `--wave-id`
   - resolves threshold fixture path from wave id
   - preserves wave-19 max-runtime override behavior
3. Replace existing wave1-wave19 checker script bodies with symlink entrypoints.
4. Run wave checker contract tests and full CI tools regression.

## Affected Modules

- `scripts/ci/check_non_kolme_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/check_non_kolme_wave1_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave2_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave3_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave4_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave5_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave6_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave7_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave8_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave9_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave10_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave11_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave12_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave13_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave14_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave15_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave16_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave17_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave18_wrapper_family_budget_trend.sh`
- `scripts/ci/check_non_kolme_wave19_wrapper_family_budget_trend.sh`
- `scripts/ci/test_non_kolme_wave_budget_trend_runner_contract.sh`

## Risks / Mitigations

- Risk: checker entrypoints lose compatibility with existing CI command surface.
  Mitigation: keep all wave1-wave19 checker filenames as symlinks and verify via existing wave test wrappers plus CI tools suite.
- Risk: wave-19 runtime budget behavior regresses.
  Mitigation: preserve explicit `KAMN_NON_KOLME_WAVE19_TREND_MAX_SECONDS` handling in shared runner.
- Risk: threshold path resolution drift.
  Mitigation: explicit threshold fixture existence checks keyed by resolved wave id.

## Interfaces / Contracts

- Preserve checker interface:
  - `bash scripts/ci/check_non_kolme_wave${wave}_wrapper_family_budget_trend.sh ...`
- Preserve downstream checker output contracts from `kolme_wrapper_inventory_baseline.py`:
  - `status=pass|fail`
  - `mode=trend`
  - deterministic reason taxonomy/value markers

## ADR

No ADR required; this is script deduplication with no protocol/dependency/architecture change.
