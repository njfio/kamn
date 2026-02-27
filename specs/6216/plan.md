# Plan: Issue 6216 - Publish Channel/Task/Bridge Lifecycle Events via WebSocket Fanout

- Issue: #6216
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Extend `ServiceApiWebsocketEventFanout` with typed publish helpers for:
   - channel create
   - task submit and task transitions
   - bridge submit and bridge forward
2. Wire middleware success paths to invoke the new publish helpers.
3. Add unit regression in `websocket.rs` validating event names and sequence progression.
4. Run scoped verification for `kamn-node`.

## Affected Modules

- `crates/kamn-node/src/service_api_endpoint/websocket.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs`

## Risks and Mitigations

1. Risk: event schema drift could break consumers.
   - Mitigation: keep event payload fields minimal and deterministic; add regression assertions on event names and sequence.
2. Risk: accidental duplicate publication.
   - Mitigation: publish only on successful persistence branches.
