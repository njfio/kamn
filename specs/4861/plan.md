# Plan — Issue #4861

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`
- `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/ci/check_kolme_wave_wrapper_family_budget_trend_impl.sh`
- `scripts/ci/test_kolme_wave_budget_trend_runner_contract.sh`
- `scripts/lib/test_harness.sh`
- `scripts/lib/test_test_harness_migration_contract.sh`
- `scripts/lib/write_json_file.sh`
- `scripts/lib/test_json_write_helper_migration_contract.sh`
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`

## Risks / Mitigations

- Risk: migration drift or hidden coupling across scripts/wrappers/checkers.
  Mitigation: phased rollout with compatibility checks and deterministic regression lanes.
- Risk: CI runtime growth.
  Mitigation: retain bounded fast-gate budgets and enforce explicit threshold checks.

## Interfaces / Contracts

- Preserve existing lane entrypoint compatibility unless explicitly versioned.
- Keep reason taxonomy/version markers deterministic and fail closed on drift.

## ADR

- Required if implementation introduces architecture/dependency/protocol strategy changes.
