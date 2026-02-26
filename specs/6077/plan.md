# Plan: Issue #6077

## Approach
1. Add a regression test that proves daemon relay processing fails closed when no forwarding route exists:
   - relay spool entries remain durable,
   - sender message state stays `created`.
2. Update daemon relay tick logic to only project relayed status for entries that were actually forwarded successfully.
3. Extend live service API integration coverage with a retry flow:
   - send while recipient unavailable,
   - daemon attempt requeues pending entry,
   - recipient comes online,
   - daemon rerun forwards successfully,
   - recipient observes delivery and restart preserves delivered state.
4. Add/update architecture documentation for the runtime delivery lifecycle and relay route-map dependency.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/daemon_relay_projection_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/architecture/service-api-delivery-flow.md`
- `specs/milestones/r66-r57-residual-gap-closure/index.md`

## Risks / Mitigations
- Risk: Existing tests currently assume synthetic relay projection.
  Mitigation: Rewrite those tests to enforce fail-closed pending semantics and real forward execution.
- Risk: Retry integration could be flaky due to endpoint readiness races.
  Mitigation: Use deterministic loopback address reservation and existing readiness waits before daemon forwarding pass.
- Risk: State/projection writes could diverge between sender and recipient stores.
  Mitigation: Assert both state files and relay spool files at each phase boundary.

## Interfaces / Contracts
- Route contracts unchanged:
  - `POST /v1/messages/send`
  - `POST /v1/messages/relay`
  - `GET /v1/channels/recipient:{did}/messages`
  - `GET /v1/messages/{id}`
- Runtime behavioral contract tightened:
  - recipient-targeted entries are only projected to `relayed` after successful forward execution.
