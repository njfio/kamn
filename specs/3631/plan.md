# Plan - Issue #3631

## Approach

1. Validate deterministic flaky reproducer and capture/report suites.
2. Validate root-cause recurrence and quarantine cleanup governance suites.
3. Validate anti-flake merge-gate and evidence/report governance suites.
4. Close story with AC-mapped lifecycle artifacts.

## Affected Paths

- `specs/3631/spec.md`
- `specs/3631/plan.md`
- `specs/3631/tasks.md`

## Risks / Mitigations

- Risk: partial anti-flake closure can leave deterministic evidence gaps in merge governance.
  Mitigation: require reproducer, recurrence, metadata, and merge-policy suites in story verification.

## ADR

- Not required (lifecycle artifact closure only).
