# Milestone R67 - Runtime Hardening and Surface Reduction

- Milestone: `r67-runtime-hardening-and-surface-reduction`
- Epic: #6083
- Stories: #6084, #6085, #6086
- Tasks: #6087, #6088, #6089

## Problem Frame
The repository currently has no open issues, but the next highest-impact risk remains concentrated in three lanes: production panic-surface enforcement, phase-1 `kamn-core` decomposition, and shell-surface reduction. This milestone executes these lanes in sequence with explicit non-regression evidence.

## Execution Order (Dependency-Aware)
1. Production panic-surface zero-regression: #6084 -> #6087
2. `kamn-core` decomposition phase 1: #6085 -> #6088
3. Shell-surface reduction wave 1: #6086 -> #6089

## Artifact Index
- #6083: (epic issue; lifecycle tracked in issue thread)
- #6084: (story issue; lifecycle tracked in issue thread)
- #6085: (story issue; lifecycle tracked in issue thread)
- #6086: (story issue; lifecycle tracked in issue thread)
- #6087: `specs/6087/spec.md`, `specs/6087/plan.md`, `specs/6087/tasks.md`
- #6088: `specs/6088/spec.md`, `specs/6088/plan.md`, `specs/6088/tasks.md`
- #6089: (pending artifacts)

## Exit Criteria
1. All three lanes are merged in dependency order.
2. Panic-surface, decomposition, and shell-surface deltas are evidenced in closure comments.
3. Story/task specs reach `Implemented` with AC/test traceability.
