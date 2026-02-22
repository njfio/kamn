# Issue #5543 Tasks - Service API Scope-Policy Fixture Exclusive Allow-Only/Deny-Only Route Coverage Metrics Exposure

## Ordered Tasks
1. T1 (tests/red): extend four endpoint lanes with failing assertions for allow-only and deny-only markers.
2. T2 (impl/green): add exclusive route-count derivation in fixture projection and snapshot wiring.
3. T3 (impl/green): emit markers in `/metrics` payload.
4. T4 (verify): run targeted + scoped tests, extraction contract, fmt, clippy.
5. T5 (docs/spec): update spec status to Implemented and complete issue/PR closure artifacts.

## Test Tier Mapping
- Unit: unit observability metrics lane
- Functional: route contract metrics lane
- Conformance: C-01..C-04 mapped to four lanes
- Integration: HTTP + TLS lanes
- Regression: scoped endpoint suite
- Performance: existing performance lane in scoped endpoint suite
- Property/Contract/Snapshot/Fuzz/Mutation: N/A with PR justification
