# Plan - Issue #3798

## Approach

1. Validate quarantine-lane capture behavior.
2. Validate deterministic flaky summary/report marker checks.
3. Close subtask with AC-mapped lifecycle artifacts.

## Affected Paths

- `specs/3798/spec.md`
- `specs/3798/plan.md`
- `specs/3798/tasks.md`

## Risks / Mitigations

- Risk: missing summary determinism reduces flaky triage auditability.
  Mitigation: bind closure to report-comment and registry-sync suites.

## ADR

- Not required (lifecycle artifact closure only).
