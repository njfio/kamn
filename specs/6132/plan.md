# Plan: Issue #6132

## Approach
1. Add/extend unit tests in `watchdog.rs` to lock deterministic integer-floor ratio behavior at threshold boundaries.
2. Replace the `f64` percentage computation with integer math using a safe widened intermediate (`u128`).
3. Keep function signatures and output payloads unchanged.
4. Run scoped formatting/lint/tests for `kamn-runtime-guards`.

## Affected Modules
- `crates/kamn-runtime-guards/src/watchdog.rs`

## Risks
- Risk: accidental semantic drift at threshold boundaries.
  - Mitigation: add explicit boundary tests (`2/3` at threshold `67`; `7/10` at threshold `70`).
- Risk: overflow in multiplication if done in native width.
  - Mitigation: perform arithmetic with `u128` intermediate.

## Interfaces/Contracts
- No public API changes.
- Internal behavior contract change: percentage computation implementation is integer-only while preserving floor semantics.
