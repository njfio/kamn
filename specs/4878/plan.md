# Plan — Issue #4878

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/lib/test_test_harness_migration_contract.sh`
- Migrated test families across:
  - `scripts/bridge/test_*.sh`
  - `scripts/canary/test_*.sh`
  - `scripts/channel/test_*.sh`
  - `scripts/ci/test_*.sh`
  - `scripts/compliance/test_*.sh`
  - `scripts/cutover/test_*.sh`
  - `scripts/dashboard/test_*.sh`
  - `scripts/deploy/test_*.sh`

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
