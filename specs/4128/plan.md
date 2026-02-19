# Issue #4128 Plan

- Issue: #4128
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Approach
1. Confirm downstream story/task/subtask closures (`#4129` and `#4130` chains).
2. Backfill missing epic/story/task specs for traceable AC-to-test mappings.
3. Run representative property/fuzz/concurrency test commands that cover each AC axis.
4. Close open epic/story issues once evidence is captured.

## Affected Files
- `specs/4128/{spec.md,plan.md,tasks.md}`
- `specs/4129/{spec.md,plan.md,tasks.md}`
- `specs/4130/{spec.md,plan.md,tasks.md}`
- `specs/4131/{spec.md,plan.md,tasks.md}`
- `specs/4132/{spec.md,plan.md,tasks.md}`
- `specs/4138/spec.md`

## Risks and Mitigations
- Risk: backfill-only closure could drift from implemented behavior.
  - Mitigation: map every AC to already-implemented passing tests.
- Risk: spec-state inconsistency (closed issues with non-Implemented spec status).
  - Mitigation: normalize status markers during backfill.

## Interfaces / Contracts
- Property invariant seed policy and helper contracts.
- Parser fuzz provenance/taxonomy contracts.
- Concurrency CI-smoke/local-heavy boundary and selector contracts.
