# Issue #5547 Tasks - Service API Scope-Policy Fixture Method Overlap and Exclusive Coverage Metrics Exposure

## Ordered Tasks
1. T1 (tests/red): add failing marker assertions for overlap/allow-only/deny-only method counts in four lanes.
2. T2 (impl/green): add method set-arithmetic projection + snapshot wiring.
3. T3 (impl/green): emit method markers in metrics payload.
4. T4 (verify): run targeted lanes, scoped suite, extraction contract, fmt, clippy.
5. T5 (docs/spec): mark spec implemented and finalize issue/PR closure artifacts.

## Test Tier Mapping
- Unit: unit observability metrics lane
- Functional: route contract metrics lane
- Conformance: C-01..C-05 mapped to lane assertions
- Integration: HTTP + TLS lanes
- Regression: scoped endpoint suite
- Performance: existing performance lane in scoped suite
- Property/Contract/Snapshot/Fuzz/Mutation: N/A with PR justification
