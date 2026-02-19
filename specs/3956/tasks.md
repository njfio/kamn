# Issue #3956 Tasks

- Issue: #3956
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add failing freshness-outcome and stale non-failover regression tests.
- [x] T2 (Green): implement typed rotation freshness outcomes and fail-closed non-failover stale rejection.
- [x] T3 (Green): update signer reason-taxonomy/source docs contracts for new freshness marker.
- [x] T4 (Green): update runtime-watchdog attestation docs markers and docs-contract assertions.
- [x] T5 (Regression): run targeted signer/docs contract suite, `cargo fmt --check`, and `cargo clippy -p kamn-node -- -D warnings`.
- [ ] T6 (Verify): update issue body/labels/process log and parent child-backlog marker.

## Tier Mapping
- Unit: freshness outcome helper matrix.
- Functional: stale/fresh preflight behavior.
- Integration: main test harness stale metadata drill.
- Regression: signer taxonomy and stale reason-code drift guards.
- Performance: N/A (bounded constant-time comparison checks only).
