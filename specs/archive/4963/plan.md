# Issue #4963 Plan

- Issue: #4963
- Status: Implemented

## Approach
- Run first archive migration wave and publish archive index report.
- Extend archive-policy checker with index/report parity requirements.
- Add regression tests for parity drift scenarios.

## Affected Modules
- `specs/archive/index.md`
- `scripts/ci/check_spec_archive_policy.sh`
- `scripts/ci/test_check_spec_archive_policy.sh`

## Risks and Mitigations
- Risk: index/report mismatch allows silent archive drift.
- Mitigation: fail-closed parity checks and regression tests.

## Interface Contract
- Preserve archive index/report marker keys and reason-taxonomy output.

## ADR
- Not required.
