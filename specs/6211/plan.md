# Plan: Issue 6211 - Replace Censorship Ratio f64 Arithmetic with Integer Math

- Issue: #6211
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extract watchdog ratio computation into a helper using integer arithmetic:
   `(delivered * 100) / expected` with widened integer type.
2. Replace `f64` ratio logic in `observe_gossip` with the helper.
3. Add regressions that assert:
   - floor behavior on fractional ratios
   - bounded behavior on large valid values
4. Run scoped format/lint/tests for `kamn-runtime-guards`.

## Affected Modules

- `crates/kamn-runtime-guards/src/watchdog.rs`

## Risks and Mitigations

1. Risk: accidental rounding behavior drift.
   - Mitigation: explicit floor behavior regression tests.
2. Risk: overflow with large `usize` values.
   - Mitigation: widen to `u128` in helper before multiply.
