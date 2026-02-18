# Plan — Issue #4859

## Approach

- Start with failing spec-derived checks for target conformance cases.
- Implement the smallest deterministic change that satisfies ACs.
- Preserve fast-gate budget and compatibility contracts while reducing shell-surface duplication where applicable.

## Affected Modules

- `docs/plans/2026-02-17-shell-loc-reduction-plan.md`
- `specs/milestones/r27-43-shell-loc-maintainability-and-shell-to-rust-ratio-sustainment-governance/index.md`
- `scripts/lib/exec_dispatch.py`
- `scripts/lib/test_exec_dispatch_registry.sh`
- `scripts/lib/test_harness.sh`
- `scripts/lib/write_json_file.sh`
- `scripts/framework/declarative_policy_checker.py`
- `scripts/framework/generate_lane_artifacts.py`
- `scripts/framework/check_lane_registry_drift.sh`
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
