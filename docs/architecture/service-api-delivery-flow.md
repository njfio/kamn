# Service API Delivery Flow

schema_version=kamn.docs.service-api-delivery-flow.v1
last_updated=2026-02-26

## Purpose
This document defines how recipient-targeted messages move from `POST /v1/messages/send` to recipient-visible delivery state, including retry and restart behavior.

## Runtime Components
- Send ingress: `POST /v1/messages/send`
- Relay spool: `KAMN_SERVICE_API_RELAY_SPOOL_FILE` NDJSON queue
- Sender durable state: `KAMN_SERVICE_API_STATE_FILE`
- Daemon relay processor: runtime mode `daemon`
- Recipient relay ingest: `POST /v1/messages/relay`
- Recipient mailbox/message reads:
  - `GET /v1/channels/recipient:{did}/messages`
  - `GET /v1/messages/{id}`

## Delivery Lifecycle
1. Sender issues `POST /v1/messages/send`.
2. Service API persists message state as `created` and appends a relay spool entry.
3. Daemon relay tick drains spool entries and attempts HTTP forward to recipient route map:
   - recipient route map env: `KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON`
4. On successful forward (`/v1/messages/relay` returns 2xx):
   - sender state projection updates `created -> relayed`.
5. Recipient runtime upserts relayed message and mailbox projection.
6. Recipient `GET /v1/messages/{id}` transitions `relayed -> delivered` for the authenticated recipient DID.

## Fail-Closed Semantics
- If recipient forwarding is unavailable (no route map or failed recipient connect), daemon must not project sender state to `relayed`.
- Pending entries remain durable in relay spool for retry.
- Sender message status remains `created` until a successful forward occurs.

## Retry And Restart Behavior
- Forwarding failures are requeued in relay spool and retried on later daemon ticks.
- Once recipient endpoint is available, a subsequent daemon run forwards pending entries and projects sender state.
- Recipient delivered state persists across service API restart via the same durable state file.

## Verification Coverage
Primary integration and regression coverage:
- `integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract`
- `integration_service_api_endpoint_cross_node_relay_delivery_contract`
- `regression_daemon_relay_tick_loop_without_route_map_preserves_pending_relay_entries`
- `integration_runtime_daemon_without_route_map_preserves_relay_spool_entries`
