# Issue 5895 Plan

- Issue: #5895

## Approach
1. Introduce an explicit daemon tick processing loop that executes for each computed daemon tick and performs relay spool drain + state projection each cycle.
2. Capture runtime processing telemetry (tick duration and relay processing outcomes) and project it into daemon observability fields.
3. Extend websocket route behavior from single-frame send to bounded persistent stream behavior with deterministic frame ordering.
4. Validate lifecycle transitions across send/query/websocket integration tests and update existing synthetic expectation tests accordingly.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`

## Risks and Mitigations
- Risk: Existing deterministic tests assert hardcoded observability numbers.
  - Mitigation: Update tests to validate runtime-derived invariants and bounded ranges rather than fixed synthetic constants.
- Risk: Persistent websocket stream can stall request-budget completion.
  - Mitigation: Use deterministic bounded frame contract and explicit close semantics.
- Risk: Tick-loop activation changes shutdown timing behavior.
  - Mitigation: Keep shutdown contract source-of-truth via existing completion evaluator and assert bounded tick count behavior.

## Interface/Contract Notes
- No API path changes.
- `/v1/events/ws` contract changes from one-frame to multi-frame stream semantics.
- `/metrics` remains Prometheus text format; observability values change to runtime-measured derivation.

## ADR
- No ADR required (implementation hardening within existing architecture).
