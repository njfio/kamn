# Plan — Issue #4876

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- `scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh`
- `scripts/framework/wave_definitions/non_kolme_wave19_lightweight_wrappers.txt`
- `scripts/ci/test_ci_tools.sh`
- `docs/ci/strategy.md`

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
