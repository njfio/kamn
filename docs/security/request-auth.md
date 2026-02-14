# Request Signature Auth (Phase 2.2 Slice)

## Scope

This document captures the first request-auth middleware slice for Story #2910 and Task #2911.

Runtime mode: `api`

## Protected Routes

Auth is enforced on all service API routes except:

- `GET /healthz`
- `GET /metrics`

Protected routes fail closed unless the request includes:

- `X-KAMN-Sender-DID`
- `X-KAMN-Request-Nonce`
- `X-KAMN-Request-Signature`

## Verification Contract

Signature verification uses baseline profile matching:

- Signature format: `sig:ed25519:baseline-v1:<sender>:<nonce>:<state_hash>:<payload_len>`
- State hash: `service-api:<chain_id>:<chain_version>`
- Payload input: raw request body

Request rejections:

- `401 Unauthorized` for missing headers, invalid sender DID, invalid nonce, or signature mismatch.
- `409 Conflict` for replayed `<sender_did, nonce>` pairs.

## Validation

Low-cost local commands:

- `cargo test -p kamn-node service_api_endpoint -- --nocapture`
- `cargo test -p kamn-node service_api_endpoint_rejects_ -- --nocapture`
- `cargo test -p kamn-node`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`

Live lane compatibility:

- `bash scripts/runtime/test_validate_service_api_live.sh`
- `bash scripts/runtime/validate_service_api_live.sh`
- `bash scripts/runtime/test_validate_service_api_request_auth_live.sh`
- `bash scripts/runtime/validate_service_api_request_auth_live.sh`

## Live Validation Evidence

Task and subtask evidence:

- Task: #2913
- Subtask: #2914

Deterministic success markers from `validate_service_api_request_auth_live.sh`:

- `status=pass`
- `final_decision=GO`
- `unauthorized_guard_status=verified`
- `replay_guard_status=verified`
- `probe_status=verified`

Deterministic fail-closed marker example:

- `bash scripts/runtime/validate_service_api_request_auth_live.sh --max-seconds nope`
- stderr marker: `max-seconds must be an integer`

## TDD Evidence

Red command:

- `cargo test -p kamn-node service_api_endpoint_rejects_ -- --nocapture`

Red failure markers before implementation:

- expected `401 Unauthorized` assertion failed for missing-auth request.
- expected `409 Conflict` assertion failed for replayed nonce request.

Green command:

- `cargo test -p kamn-node service_api_endpoint_rejects_ -- --nocapture`

Green marker:

- both auth/replay tests pass with deterministic status/body checks.
