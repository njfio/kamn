# Plan: Issue #6127

## Approach
1. Introduce a canonical static transition-edge table for task lifecycle rules.
2. Refactor `next_state` and `transition_between` to query the same table.
3. Add regression assertions for uniqueness and bidirectional consistency.
4. Run scoped lifecycle tests and quality gates (`fmt`, scoped `clippy`).

## Affected Modules
- `crates/kamn-core/src/task_lifecycle.rs`
- `specs/6127/spec.md`
- `specs/6127/plan.md`
- `specs/6127/tasks.md`

## Risks / Mitigations
- Risk: table entry omissions could break existing valid transitions.
  Mitigation: regression test iterates all table entries and existing lifecycle tests remain in scope.
- Risk: accidental duplicate rows in canonical table.
  Mitigation: uniqueness assertions over `(from, transition)` and `(from, to)` pairs.

## Interfaces / Contracts
- No public API shape changes.
- Behavior contract preserved; only internal transition rule representation is consolidated.
