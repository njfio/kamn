# Tasks: Issue #6111

## Ordered Tasks
- T1 (RED): Add websocket frame parser tests for inline/16-bit/64-bit/truncated/overflow cases.
- T2 (GREEN): Implement RFC 6455 extended payload-length decoding in SDK websocket frame parsing.
- T3 (GREEN): Add integration coverage for `read_event_once` consuming an extended-length frame (>125 bytes).
- T4 (VERIFY): Run targeted tests and quality gates (`fmt`, `clippy`, `cargo test -p kamn-sdk`).

## Tier Mapping
- Unit: T1, T2
- Functional: T3
- Conformance: T1, T3
- Integration: T3
- Regression: T4
- Property: N/A (deterministic frame parsing paths)
- Contract/DbC: N/A (no DbC macros in crate)
- Snapshot: N/A (no snapshots)
- Fuzz: N/A (no new fuzz target in this issue scope)
- Mutation: N/A (workspace mutation gate handled in CI)
- Performance: N/A (no throughput/latency contract change)
