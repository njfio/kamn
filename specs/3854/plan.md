# Plan - Issue #3854

## Approach

1. Validate threshold-policy and reason-taxonomy behavior via existing deterministic checker tests.
2. Capture deterministic fail-closed scenarios in lifecycle artifacts.
3. Close subtask with explicit AC-to-test traceability.

## Affected Paths

- `specs/3854/spec.md`
- `specs/3854/plan.md`
- `specs/3854/tasks.md`

## Risks / Mitigations

- Risk: threshold-policy regression introduces nondeterministic failures.
  Mitigation: keep fail-mode contract tests mapped to deterministic reason markers.

## ADR

- Not required (lifecycle artifact closure only).
