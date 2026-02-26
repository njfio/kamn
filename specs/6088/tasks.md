# Tasks: Issue #6088

## Ordered Tasks
- T1 (RED/Conformance): Run `cargo test -p kamn-live-probe-matrix` before crate creation and capture expected failure.
- T2 (Implementation): Add `crates/kamn-live-probe-matrix` with extracted `live_probe_matrix` implementation and crate-local tests.
- T3 (Implementation/Regression): Convert `crates/kamn-core/src/live_probe_matrix.rs` to compatibility re-export; add crate dependency wiring.
- T4 (Verification): Run `cargo test -p kamn-live-probe-matrix` and `cargo test -p kamn-core --test live_probe_matrix_contract`.
- T5 (Verification): Run `cargo test -p kamn-core` to ensure no broader `kamn-core` regressions.

## Tier Mapping
- Unit: T2, T4
- Functional: T4
- Integration: T4, T5
- Regression: T3, T4, T5
- Conformance: T1, T4
