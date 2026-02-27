# Tasks: Issue #6132

- T1 (Red): Add conformance tests for integer-floor ratio behavior at boundary conditions.
- T2 (Green): Replace `f64` ratio computation in `observe_gossip` with integer arithmetic using widened intermediates.
- T3 (Regression): Re-run existing watchdog tests to verify no behavior regression outside ratio math.
- T4 (Verify): Run scoped fmt/clippy/tests and record AC/C-case evidence in PR.
