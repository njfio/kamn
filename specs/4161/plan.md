# Plan - Issue #4161

## Approach

1. Capture parent-task lifecycle artifacts for already-merged zeroization subtasks (`#4165`, `#4166`).
2. Re-run signer zeroization and docs-contract tests that map directly to parent ACs.
3. Close parent task with AC->conformance traceability and test-tier coverage in PR.

## Affected Paths

- `specs/4161/spec.md`
- `specs/4161/plan.md`
- `specs/4161/tasks.md`

## Risks / Mitigations

- Risk: parent task remains open without in-repo lifecycle artifacts despite merged implementation.
  Mitigation: add missing artifacts and tie ACs to deterministic, existing tests.

- Risk: future signer changes could regress zeroization semantics.
  Mitigation: retain and re-verify regression tests plus docs-contract markers.

## ADR

- Not required (task artifact closure and validation only).
