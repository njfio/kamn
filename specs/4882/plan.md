# Plan — Issue #4882

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `scripts/framework/lane_registry.json`
- `scripts/framework/generate_lane_artifacts.py`
- `scripts/framework/check_lane_registry_drift.sh`
- `scripts/framework/test_lane_registry_generation.sh`
- `scripts/framework/test_check_lane_registry_drift.sh`
- `scripts/framework/test_contract_framework.sh`
- `docs/architecture/lane-registry-generation.md`
- `docs/architecture/adr-lane-registry-source-of-truth.md`

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
