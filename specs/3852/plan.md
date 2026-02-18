# Plan - Issue #3852

## Approach

1. Validate subtask-delivered behavior for combined report generation and threshold policy enforcement.
2. Validate composed ignored-test/script trend governance lane and harness budget contract lane.
3. Record closure artifacts with explicit AC/conformance mapping.

## Affected Paths

- `specs/3852/spec.md`
- `specs/3852/plan.md`
- `specs/3852/tasks.md`

## Risks / Mitigations

- Risk: task remains open despite implemented governance checks, causing backlog drift.
  Mitigation: consolidate deterministic verification evidence and close with lifecycle artifacts.

- Risk: partial validation misses one governance surface.
  Mitigation: include generator, checker, ignored/script lane, and harness lane checks.

## ADR

- Not required (task closure and verification only).
