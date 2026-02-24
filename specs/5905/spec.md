# Spec: Issue #5905 - Persistent Service API WebSocket Streaming with Live Event Delivery

- Issue: #5905
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
The service API websocket route currently sends a fixed payload/frame set and then closes the socket. This is fire-and-forget behavior, not a persistent event stream. Clients cannot maintain subscriptions to receive live delivery events.

## Scope
In scope:
- Keep websocket sessions open until client disconnect or runtime shutdown.
- Add runtime websocket event fan-out for service API lifecycle updates.
- Emit live message-created lifecycle events on websocket streams after upgrade.
- Maintain existing websocket auth/scope and rejection reason-code behavior.
- Add integration/regression tests that require multi-frame delivery over a single websocket session.

Out of scope:
- Cross-node federation.
- External queue/broker systems.
- Backfilling historical events to newly connected sockets.

## Acceptance Criteria
### AC-1 WebSocket sessions are persistent
Given a successful websocket upgrade,
When the server has no immediate events,
Then the connection remains open and server does not immediately close the socket.

### AC-2 Live events are pushed to active sessions
Given an active websocket subscription and subsequent service API message traffic,
When messages are created/updated,
Then corresponding websocket event frames are delivered on that same session.

### AC-3 Existing validation remains fail-closed
Given invalid websocket headers/scope/auth conditions,
When request validation fails,
Then behavior remains fail-closed with existing reason-code taxonomy.

### AC-4 Regression guard prevents one-shot fallback
Given future refactors,
When websocket stream logic regresses to one-shot close behavior,
Then regression tests fail.

## Conformance Cases
- C-01 (AC-1, Integration): websocket stream remains open after initial frame and supports continued reads.
- C-02 (AC-2, Integration): message send request after websocket upgrade produces follow-up websocket event frame.
- C-03 (AC-3, Regression): websocket validation rejection paths preserve reason codes and status mapping.
- C-04 (AC-4, Regression): stream test fails if server emits close immediately after first frame.

## Success Metrics / Observable Signals
- Websocket sessions no longer terminate immediately after upgrade in normal operation.
- Service API message traffic is observable through websocket frames without reconnecting.
- Updated websocket integration/regression tests pass and fail closed on one-shot regressions.

## Spec Verification (AC -> Tests)
- AC-1: `integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event`
- AC-2: `regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade`
- AC-3: `regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers`; `regression_service_api_endpoint_websocket_rejects_invalid_version_header`; `regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header`; `regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode`
- AC-4: `regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade`
