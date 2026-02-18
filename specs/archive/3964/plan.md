# Plan - Issue #3964

## Approach

1. Capture story-level contract mapping across child tasks:
   - migration/parity (`#3966`),
   - shell-surface governance (`#3967`).
2. Verify representative wrapper migration, parity, and shell-budget contract commands.
3. Commit story lifecycle artifacts so closure evidence is preserved in-repo.
4. Close story with AC->test matrix and conformance totals.

## Affected Paths

- `specs/3964/spec.md`
- `specs/3964/plan.md`
- `specs/3964/tasks.md`

## Risks / Mitigations

- Risk: child task completion is not reflected at story level, leaving audit gaps.
  Mitigation: explicit story-level AC mapping to deterministic command checks.

- Risk: shell-surface governance drifts while migration parity remains green.
  Mitigation: include duplication + ratio + hard ceiling + budget delta checks in story conformance set.

## ADR

- Not required (story closure artifacts and validation mapping only).
