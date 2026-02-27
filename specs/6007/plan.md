# Plan: Issue #6007

## Approach
1. Map existing relay processing and observability projection paths in `kamn-node` runtime execution flow.
2. Add RED conformance tests for runtime relay progression and metrics advancement.
3. Wire daemon tick execution to relay spool processing and aggregate deterministic counters into runtime observability output.
4. Connect metrics rendering to live relay counters sourced from runtime state.
5. Run scoped regression suite for full-supervisor and service API contracts.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/runtime_observability.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/*.rs`

## Risks and Mitigations
- Risk: Runtime liveness tests can flake under parallel execution due shared ports.
  Mitigation: use deterministic per-test bind addresses and serialize critical integration slices where needed.
- Risk: Metrics assertions become brittle if unrelated counters change.
  Mitigation: assert invariant-specific relay counters and reason-code markers, not entire payload snapshots.

## Interface / Contract Notes
- Preserve existing runtime report field names and reason-code taxonomy.
- No wire-format changes; only runtime source-of-truth wiring for existing counters.

## ADR
- Not required: no new dependency, protocol, or cross-crate architecture policy change.
