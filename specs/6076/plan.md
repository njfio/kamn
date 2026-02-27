# Plan: Issue #6076

## Approach
1. Consume merged child-task implementation evidence from #6077 (PR #6078) as the concrete runtime delivery slice.
2. Verify story AC traceability from send ingress through relay forwarding and recipient delivery/read transitions.
3. Confirm restart durability and fail-closed no-route behavior through existing runtime/service integration tests.
4. Complete lifecycle closure updates (spec status markers, issue status, and process log).

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/daemon_relay_projection_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/architecture/service-api-delivery-flow.md`
- `specs/6077/spec.md`

## Risks / Mitigations
- Risk: story closure drifts from merged behavior if AC mapping is implicit.
  Mitigation: explicit AC->test mapping in PR body and issue closure comment.
- Risk: restart or async test nondeterminism after upstream rebase.
  Mitigation: keep sender nonce domains isolated and align projection assertions to fail-closed semantics.

## Interfaces / Contracts
- `POST /v1/messages/send`
- `POST /v1/messages/relay`
- `GET /v1/channels/recipient:{did}/messages`
- `GET /v1/messages/{id}`
- Runtime contract: no-route relay attempts retain pending spool/state; successful forwarding gates `created -> relayed`.
