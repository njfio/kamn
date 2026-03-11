# Runtime Capability Audit R57

## Status Taxonomy
- implemented_and_wired: real entrypoint and executable behavior are present on current main
- gated_or_partial: runtime path exists but is feature-gated, manual, or incomplete for full product claims
- contract_only: contracts/docs exist without enough real wired runtime evidence for the claimed capability
- missing: no meaningful implementation path was found for the claimed capability

## Message Routing
status: gated_or_partial

evidence:
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` wires message creation via `message_store.create_message(...)`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` wires requester retrieval via `message_store.get_message_for_requester(...)`
- `crates/kamn-sdk/src/service_client_message_task_routes.rs` exposes `get_message_delivery(...)`
- `crates/kamn-sdk/src/live/agent.rs` consumes service message delivery records

assessment:
- The service API supports authenticated message creation, storage, and requester retrieval.
- The SDK can consume delivery-shaped responses from the service API.
- The reviewed path does not show autonomous peer-to-peer delivery orchestration or background delivery routing from node to node.

follow_on_issues:
- `#6883` implement real routed agent-to-agent message delivery beyond service-api storage/retrieval

## Task Dispatch
status: gated_or_partial

evidence:
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` wires task creation via `message_store.create_task(...)`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` wires manual transitions via `transition_task(task_id, "accepted")` and `transition_task(task_id, "completed")`
- `crates/kamn-core/src/task_operations.rs` is the core task-operation surface on current main

assessment:
- Task creation and manual state progression are wired.
- The reviewed path does not show autonomous worker selection, queue-based dispatch, or scheduler-driven task assignment.

follow_on_issues:
- `#6884` implement real task dispatch and worker assignment beyond manual task state transitions

## Audit Emission And Export
status: gated_or_partial

evidence:
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs` contains append-only audit-ledger construction and verification
- `crates/kamn-core/src/upgrade_orchestration.rs` exposes `audit_view()`
- `crates/kamn-core/src/redaction_compliance.rs` defines concrete audit-event surfaces
- `crates/kamn-core/README.md` references `audit_exports`

assessment:
- Concrete audit record and audit view surfaces exist.
- The reviewed path does not establish that audit export is consistently populated across major runtime flows.

follow_on_issues:
- `#6885` audit and complete runtime audit-export population across wired node flows

## Live Transport
status: gated_or_partial

evidence:
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs` contains live transport runtime logic
- `crates/kamn-core/src/p2p_transport/native_runtime.rs` contains feature-gated libp2p native runtime support
- `crates/kamn-core/src/p2p_transport/swarm_stack.rs` contains swarm wiring
- `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime.rs` exercises libp2p native adapter runtime under `libp2p-live-transport`
- `crates/kamn-core/tests/p2p_live_transport_runtime.rs` covers live transport runtime behavior

assessment:
- Live transport is not missing; it exists behind the `libp2p-live-transport` feature and has executable test coverage.
- It is still partial relative to stronger product claims about generalized multi-node coordination.

follow_on_issues:
- `#6879` stabilize SDK-direct live validation on main

## Summary
- The strongest overclaim is not “nothing works.”
- The accurate statement is that several runtime surfaces are wired, but message routing, task dispatch, and audit export remain partial relative to broader coordination-network claims.
