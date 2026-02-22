# Issue #5541 Tasks - Service API Scope-Policy Fixture Allow/Deny Overlap Route Coverage Metrics Exposure

## Ordered Tasks
1. T1 (tests/red): extend endpoint metrics lane assertions for overlap marker/value in:
   - functional lane
   - HTTP integration lane
   - TLS integration lane
   - unit observability lane
2. T2 (impl/green): add overlap-route projection and snapshot wiring.
3. T3 (impl/green): emit overlap marker in `/metrics` payload rendering.
4. T4 (verify): run scoped tests, module extraction contract test, fmt, and clippy gates.
5. T5 (docs/spec): set `specs/5541/spec.md` status to `Implemented`, finalize issue/PR closure metadata.

## Test Tier Mapping
- Unit: endpoint unit observability metrics test
- Functional: endpoint route contracts test
- Conformance: C-01..C-04 mapped to four endpoint lanes
- Integration: HTTP + TLS endpoint metrics lanes
- Regression: scoped endpoint suite run
- Performance: scoped endpoint suite includes existing service-api performance lane
- Property/Contract/Snapshot/Fuzz/Mutation: N/A with explicit PR justification
