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
- `GET /healthz`
- `GET /metrics`

Response behavior is deterministic and intentionally lightweight for this phase.

## Validation

Low-cost local validation commands:

- `cargo test -p kamn-node service_api_endpoint -- --nocapture`
- `cargo test -p kamn-node`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`

These checks cover unit, functional, integration, and regression behavior for the API ingress module without requiring external infrastructure.
