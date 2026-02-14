# Service HTTP API (Phase 2 Slice)

## Scope

This document captures the first executable HTTP ingress slice for Story #2905 and Task #2906.

Runtime mode: `api`

## Runtime Controls

- `--runtime-mode api`
- `--api-bind <host:port>`
- `--api-max-requests <n>` (default: `1`)
- `--api-idle-timeout-ms <ms>` (default: `5000`)

Fail-closed behavior:

- `runtime-mode api` without `--api-bind` is rejected.
- `--api-max-requests` and `--api-idle-timeout-ms` must be positive integers.

## Endpoints

Implemented route contract for local deterministic ingress:

- `POST /v1/messages/send`
- `GET /v1/messages/{id}`
- `POST /v1/channels/create`
- `GET /v1/channels/{id}/messages`
- `POST /v1/tasks/create`
- `GET /v1/tasks/{id}`
- `GET /v1/agents/{did}`
- `GET /v1/events/ws`
- `GET /healthz`
- `GET /metrics`

Response behavior is deterministic and intentionally lightweight for this phase.

## Request Auth

Protected routes require deterministic request-envelope headers:

- `X-KAMN-Sender-DID`
- `X-KAMN-Request-Nonce`
- `X-KAMN-Request-Signature`

Auth policy:

- `GET /healthz` and `GET /metrics` are intentionally unauthenticated for probes/scrapes.
- Other routes fail closed with `401 Unauthorized` when required headers are missing or signature verification fails.
- Nonce replay per sender fails closed with `409 Conflict`.

Signature profile:

- Signature contract uses baseline profile matcher (`sig:ed25519:baseline-v1:...`).
- State hash input is deterministic: `service-api:<chain_id>:<chain_version>`.
- Replay key is `<sender_did, nonce>`.

## Validation

Low-cost local validation commands:

- `cargo test -p kamn-node service_api_endpoint -- --nocapture`
- `cargo test -p kamn-node`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`
- `cargo test -p kamn-node service_api_endpoint_rejects_ -- --nocapture`
- `bash scripts/runtime/test_validate_service_api_live.sh`
- `bash scripts/runtime/validate_service_api_live.sh`
- `bash scripts/runtime/test_validate_service_api_request_auth_live.sh`
- `bash scripts/runtime/validate_service_api_request_auth_live.sh`

These checks cover unit, functional, integration, and regression behavior for the API ingress module without requiring external infrastructure.

## Live Validation Evidence

Task and subtask evidence:

- Task: #2908
- Subtask: #2909

Deterministic success markers from `validate_service_api_live.sh`:

- `status=pass`
- `final_decision=GO`
- `route_contract_status=verified`
- `failure_case_status=verified`

Deterministic fail-closed marker example:

- `bash scripts/runtime/validate_service_api_live.sh --max-seconds nope`
- stderr marker: `max-seconds must be an integer`
