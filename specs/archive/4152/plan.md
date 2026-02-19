# Issue #4152 Plan

- Issue: #4152
- Status: Implemented

## Approach
1. Extend `scripts/governance/test_check_governance_lifecycle_rollback_policy.sh` with rollback trigger mismatch and taxonomy drift fixtures.
2. Add repeated-run assertions to guarantee deterministic mismatch reason ordering.
3. Keep scope test-only to minimize shell LOC growth and avoid behavior changes.

## Affected Modules
- `scripts/governance/test_check_governance_lifecycle_rollback_policy.sh`
- `specs/4152/spec.md`
- `specs/4152/plan.md`
- `specs/4152/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Reuse existing fixture/report generation pattern to keep assertions deterministic.
  - Validate explicit mismatch markers instead of broad substring checks where possible.
  - Run only targeted governance rollback test path for iteration speed and bounded shell changes.

## Interface Contract
- No new interfaces.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped subtask.
