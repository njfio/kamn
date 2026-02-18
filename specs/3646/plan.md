# Plan - Issue #3646

## Approach

1. Validate websocket anti-flake recurrence stability checks.
2. Validate flaky registry and ignored-test metadata cleanup policy checks.
3. Close task with AC-mapped lifecycle artifacts.

## Affected Paths

- `specs/3646/spec.md`
- `specs/3646/plan.md`
- `specs/3646/tasks.md`

## Risks / Mitigations

- Risk: stale quarantine metadata and flaky recurrence can silently break merge reliability.
  Mitigation: bind closure to deterministic recurrence and metadata-policy suites.

## ADR

- Not required (lifecycle artifact closure only).
