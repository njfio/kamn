# Plan - Issue #4163

## Approach

1. Add parent-task lifecycle artifacts for already-merged subtasks `#4169` and `#4170`.
2. Re-run rotation preflight policy/lane checks and docs-contract tests that enforce quorum and custody marker governance.
3. Close task with AC->conformance mapping in PR.

## Affected Paths

- `specs/4163/spec.md`
- `specs/4163/plan.md`
- `specs/4163/tasks.md`

## Risks / Mitigations

- Risk: parent task remains open/process-incomplete after subtask merges.
  Mitigation: add missing parent artifacts and deterministic command mapping.

- Risk: future edits drift custody/quorum taxonomy output contracts.
  Mitigation: preserve existing checker/docs-contract regression coverage and re-verify at closure.

## ADR

- Not required (task artifact closure and re-validation only).
