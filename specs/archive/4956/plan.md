# Issue #4956 Plan

- Issue: #4956
- Status: Implemented

## Approach
- Deliver story through tasks:
  - `#4961` policy + governance markers.
  - `#4962` archive tooling + placement checks.
  - `#4963` initial archive-wave execution + parity checks.
- Finalize story lifecycle docs after task closures.

## Affected Modules
- `docs/planning/spec-archive-policy.md`
- `scripts/ci/archive_completed_specs.py`
- `scripts/ci/check_spec_archive_policy.sh`
- `scripts/ci/test_check_spec_archive_policy.sh`
- `specs/archive/index.md`

## Risks and Mitigations
- Risk: policy/tooling/index drift.
- Mitigation: deterministic parity checks and fail-closed reason taxonomy.

## Interface Contract
- Preserve archive policy marker names and report schema fields used by checker/tests.

## ADR
- Not required.
