# Service API Contract

## Scope

This contract defines fail-closed websocket protocol/session checks for Task #4312 and Subtask #4317.

## Websocket Protocol/Session Taxonomy

- `service_api_websocket_session_reason_taxonomy_version=kamn.runtime.service-api.websocket-session-reason-taxonomy.v1`
- `service_api_websocket_session_reason_codes_csv=service_api_ws_protocol_contract_drift_detected,service_api_ws_session_frame_too_short,service_api_ws_session_frame_opcode_invalid,service_api_ws_session_frame_mask_invalid,service_api_ws_session_frame_length_mismatch,service_api_ws_session_frame_payload_utf8_invalid`

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
