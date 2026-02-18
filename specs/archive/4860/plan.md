# Plan — Issue #4860

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/lib/common.sh`
- `scripts/kolme/run_lane_dispatch.sh`
- `scripts/kolme/run_contract_lane_dispatch.sh`
- `scripts/kolme/resolve_manifest.py`
- `scripts/kolme/test_common_sh_helper_migration_contract.sh`
- `scripts/kolme/test_dispatcher_manifest_metadata_contract.sh`
- `scripts/ci/test_ci_tools.sh`
- `.ci/kolme-command-surface-asymmetry-policy.json`
- Kolme manifest metadata fixtures updated in phase scope (`wrapper_name`/`phase` entries).

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
