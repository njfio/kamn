# Issue #5545 Tasks - Service API Scope-Policy Fixture Scope Overlap and Exclusive Coverage Metrics Exposure

## Ordered Tasks
1. T1 (tests/red): add failing assertions for three new scope markers in four endpoint lanes.
2. T2 (impl/green): add scope-set overlap/exclusive projection and snapshot wiring.
3. T3 (impl/green): emit scope markers in `/metrics` payload.
4. T4 (verify): run targeted + scoped tests, extraction contract, fmt, clippy.
5. T5 (docs/spec): mark spec implemented and complete issue/PR closure artifacts.

## Test Tier Mapping
- Unit: unit observability metrics lane
- Functional: route contract metrics lane
- Conformance: C-01..C-05 mapped to lane assertions
- Integration: HTTP + TLS lanes
- Regression: scoped endpoint suite
- Performance: existing performance lane in scoped suite
- Property/Contract/Snapshot/Fuzz/Mutation: N/A with PR justification
