# Service API Contract

## Scope

This contract defines fail-closed websocket protocol/session checks for Task #4312 and Subtask #4317.

## Websocket Protocol/Session Taxonomy

- `service_api_websocket_session_reason_taxonomy_version=kamn.runtime.service-api.websocket-session-reason-taxonomy.v1`
- `service_api_websocket_session_reason_codes_csv=service_api_ws_protocol_contract_drift_detected,service_api_ws_session_frame_too_short,service_api_ws_session_frame_opcode_invalid,service_api_ws_session_frame_mask_invalid,service_api_ws_session_frame_length_mismatch,service_api_ws_session_frame_payload_utf8_invalid`

## Release Go/No-Go Protocol/Session Markers (Issue #4387)

- `service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1`
- `service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing`

Release checklist parity checks consume these markers and fail closed on drift.

## Invalid-Frame Handling Matrix

| Condition | Fail-closed reason code | Expected behavior |
|---|---|---|
| websocket contract header missing or mismatched (`X-KAMN-WebSocket-Contract != v1`) | `service_api_ws_protocol_contract_drift_detected` | reject protocol/session contract as drifted |
| websocket frame shorter than 2 bytes | `service_api_ws_session_frame_too_short` | reject session frame |
| websocket frame opcode is not single-frame text (`0x81`) | `service_api_ws_session_frame_opcode_invalid` | reject session frame |
| websocket frame is masked in server->client direction | `service_api_ws_session_frame_mask_invalid` | reject session frame |
| websocket frame payload length marker mismatches bytes present | `service_api_ws_session_frame_length_mismatch` | reject session frame |
| websocket frame payload is not utf-8 | `service_api_ws_session_frame_payload_utf8_invalid` | reject session frame |

## Validation Commands

- `cargo test -p kamn-node websocket_protocol_ -- --nocapture`
- `cargo test -p kamn-node websocket_session_ -- --nocapture`
- `cargo test -p kamn-core --test service_api_contract_docs`

## Regression

- invalid-frame and protocol-drift fail-closed behavior remains stable (`Regression: #4317`).

## Async Lifecycle Rejection Taxonomy (Issue #4316)

- `service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1`
- `service_api_lifecycle_rejection_reason_codes_csv=service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid`

## Async Lifecycle Rejection Projection Matrix

| Guardrail | Reason code projection |
|---|---|
| `async-lifecycle-limiter` | `service_api_ingress_concurrency_limit_exceeded` / `service_api_ingress_rate_limit_exceeded` |
| `sender-admission-limiter` | `service_api_ingress_sender_rate_limit_exceeded` / `service_api_ingress_sender_suspended` / `service_api_ingress_sender_duplicate_message_id` / `service_api_ingress_sender_insufficient_deposit` |
| `async-lifecycle-engine` | `service_api_ingress_anti_spam_engine_invalid` |

- Regression: #4316
