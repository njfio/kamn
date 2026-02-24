# Issue 5895 Tasks

- Issue: #5895

## Ordered Tasks

1. T1 (Tests - Red): Add/adjust integration tests for daemon per-tick relay processing and multi-frame websocket stream behavior. (Tier: Integration, Conformance)
2. T2 (Impl - Green): Implement daemon per-tick relay processing loop with runtime processing counters and state projection wiring. (Tier: Functional)
3. T3 (Impl - Green): Wire runtime-measured observability derivation into daemon/service snapshot and `/metrics` exposure. (Tier: Functional)
4. T4 (Impl - Green): Implement websocket persistent streaming helper with deterministic bounded close behavior. (Tier: Functional, Integration)
5. T5 (Refactor/Regression): Refine tests to validate runtime-derived observability invariants and preserve shutdown bound contracts. (Tier: Regression)
6. T6 (Verify): Run targeted suites and update issue/PR AC mapping evidence. (Tier: Verification)

## Tier Mapping
- Unit: helper-level parsing and stream frame sequencing invariants.
- Functional: daemon lifecycle status/metrics exposure and websocket route behavior.
- Conformance: C-01..C-05 coverage via runtime/service endpoint integration tests.
- Integration: send->daemon->query and websocket streaming path.
- Regression: shutdown-bound and lifecycle ordering non-regression coverage.
