# Plan: Issue 6206 - Unify Task Lifecycle Transition Mapping

- Issue: #6206
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Introduce canonical transition edge table in `task_lifecycle.rs`.
2. Implement `next_state` and `transition_between` as table lookups.
3. Add regressions for:
   - forward/reverse consistency on canonical edges
   - invalid edge rejection parity
4. Run scoped format/lint/tests for `kamn-core`.

## Affected Modules

- `crates/kamn-core/src/task_lifecycle.rs`

## Risks and Mitigations

1. Risk: accidental transition behavior change while refactoring.
   - Mitigation: keep canonical table identical to existing edge set and retain existing tests.
2. Risk: reverse lookup ambiguity.
   - Mitigation: keep one-to-one mapping by design; add consistency regression coverage.
