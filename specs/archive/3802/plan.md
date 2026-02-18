# Plan - Issue #3802

## Approach

1. Validate flaky registry and ignored-inventory drift checks.
2. Validate metadata-policy fail-closed behavior.
3. Close subtask with AC-mapped lifecycle artifacts.

## Affected Paths

- `specs/3802/spec.md`
- `specs/3802/plan.md`
- `specs/3802/tasks.md`

## Risks / Mitigations

- Risk: stale quarantine metadata can hide flaky regressions.
  Mitigation: require deterministic registry, drift, and metadata-policy checks in closure evidence.

## ADR

- Not required (lifecycle artifact closure only).
