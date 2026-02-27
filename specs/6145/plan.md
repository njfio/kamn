# Plan: Issue #6145

## Approach
1. Add RED tests in daemon relay orchestration for P2P success and fail/requeue behavior.
2. Introduce optional daemon relay P2P config resolution (env JSON contract) and initialize one
   live transport context per daemon execution.
3. In relay tick loop:
   - drain inbound P2P inbox frames and upsert relayed messages into service-api state
   - attempt outbound P2P forwarding first for mapped recipients
   - preserve HTTP relay forwarding as fallback path
4. Add/adjust service-api helper(s) required for daemon-side relay ingress without spinning a local
   HTTP server.
5. Run scoped `kamn-node` verification and collect AC→test evidence.

## Affected Modules
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `specs/6145/spec.md`
- `specs/6145/plan.md`
- `specs/6145/tasks.md`

## Risks / Mitigations
- Risk: P2P runtime startup can add nondeterminism to tests.
  Mitigation: keep tests deterministic with bounded waits and explicit failure messages.
- Risk: Transport send semantics can report success before recipient persistence.
  Mitigation: add recipient-side inbox drain/upsert in daemon tick and assert recipient state in
  integration coverage.
- Risk: Fallback path regressions break existing HTTP relay behavior.
  Mitigation: retain and re-run existing no-route/failed-forward regression coverage.

## Interfaces / Contracts
- New optional env contract for daemon relay P2P config:
  `KAMN_SERVICE_API_RELAY_P2P_CONFIG_JSON` (JSON object parsed by daemon runtime).
- No wire/schema changes to existing Service API routes.
