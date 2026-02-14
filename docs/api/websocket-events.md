# WebSocket Events (Phase 2.3 Slice)

## Scope

This document captures the first realtime event stream slice for Story #2915 and Task #2916.

Runtime mode: `api`

## Endpoint Contract

WebSocket route:

- `GET /v1/events/ws`

Upgrade requirements:

- `Connection` header must include `Upgrade`
- `Upgrade` header must be `websocket`
- `Sec-WebSocket-Key` header must be present and non-empty

Auth requirements:

- `X-KAMN-Sender-DID`
- `X-KAMN-Request-Nonce`
- `X-KAMN-Request-Signature`

Auth and replay enforcement reuses the request-auth middleware contract.

## Response Behavior

Successful upgrade returns:

- status line: `101 Switching Protocols`
- headers: `Upgrade: websocket`, `Connection: Upgrade`, `X-KAMN-WebSocket-Contract: v1`
- first frame: unmasked text frame containing deterministic event payload:
  - `event: state-transition`
  - runtime metadata (`runtime_mode`, `role`)
  - deterministic sequence marker (`sequence: 1`)

Fail-closed behavior:

- Missing/invalid upgrade headers return `400 Bad Request`.
- Missing/invalid auth headers or signature mismatch return `401 Unauthorized`.
- Replayed `<sender_did, nonce>` returns `409 Conflict`.

## Validation

Low-cost local commands:

- `cargo test -p kamn-node websocket -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint -- --nocapture`
- `cargo test -p kamn-node`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`
- `bash scripts/runtime/test_validate_service_api_websocket_live.sh`
- `bash scripts/runtime/validate_service_api_websocket_live.sh`

## Live Validation Evidence

Task and subtask evidence:

- Task: #2918
- Subtask: #2919

Deterministic success markers from `validate_service_api_websocket_live.sh`:

- `status=pass`
- `final_decision=GO`
- `websocket_upgrade_status=verified`
- `fail_closed_status=verified`
- `probe_status=verified`

Deterministic fail-closed marker example:

- `bash scripts/runtime/validate_service_api_websocket_live.sh --max-seconds nope`
- stderr marker: `max-seconds must be an integer`
