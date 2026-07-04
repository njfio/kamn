#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected websocket live validation script to be executable" >&2
  exit 1
fi
if ! grep -Fq "X-KAMN-Signer-Public-Key" "$VALIDATION_SCRIPT"; then
  echo "expected websocket live validation script to propagate signer public key header" >&2
  exit 1
fi
if ! grep -Fq 'return f"kamn:did:agent:pkh-{public_key_hex}"' "$VALIDATION_SCRIPT"; then
  echo "expected websocket live validation script to use self-certifying sender did" >&2
  exit 1
fi
if ! grep -Fq "KAMN_SERVICE_API_WEBSOCKET_NODE_BIN" "$VALIDATION_SCRIPT"; then
  echo "expected websocket live validation script to accept a prebuilt node binary" >&2
  exit 1
fi
if ! grep -Fq "KAMN_SERVICE_API_WEBSOCKET_SKIP_BUILD" "$VALIDATION_SCRIPT"; then
  echo "expected websocket live validation script to support skipping the default build" >&2
  exit 1
fi
if ! grep -Fq "KAMN_SERVICE_API_WEBSOCKET_PREBUILD_TARGET_DIR" "$0"; then
  echo "expected websocket live test to use an isolated prebuild target dir" >&2
  exit 1
fi
if ! grep -Fq 'timeout "$prebuild_timeout_seconds" cargo build --quiet -p kamn-node' "$0"; then
  echo "expected websocket live test to bound the isolated prebuild" >&2
  exit 1
fi
if ! grep -Fq 'runtime_storage_dir="$TMP_DIR/service-api-websocket-storage"' "$VALIDATION_SCRIPT"; then
  echo "expected websocket live validation script to use isolated runtime storage" >&2
  exit 1
fi
if ! grep -Fq -- '--storage-dir "$runtime_storage_dir"' "$VALIDATION_SCRIPT"; then
  echo "expected websocket live validation script to pass isolated storage to node" >&2
  exit 1
fi

prebuild_timeout_seconds="${KAMN_SERVICE_API_WEBSOCKET_PREBUILD_MAX_SECONDS:-900}"
if ! [[ "$prebuild_timeout_seconds" =~ ^[0-9]+$ ]] || [[ "$prebuild_timeout_seconds" -le 0 ]]; then
  echo "KAMN_SERVICE_API_WEBSOCKET_PREBUILD_MAX_SECONDS must be a positive integer" >&2
  exit 2
fi
prebuild_target_dir="${KAMN_SERVICE_API_WEBSOCKET_PREBUILD_TARGET_DIR:-$ROOT_DIR/target/service-api-websocket}"
prebuilt_node_bin="$prebuild_target_dir/debug/kamn-node"

pushd "$ROOT_DIR" >/dev/null
set +e
CARGO_TARGET_DIR="$prebuild_target_dir" timeout "$prebuild_timeout_seconds" cargo build --quiet -p kamn-node
prebuild_code=$?
set -e
popd >/dev/null
if [[ "$prebuild_code" -eq 124 ]]; then
  echo "service api websocket prebuild timed out" >&2
  exit 124
fi
if [[ "$prebuild_code" -ne 0 ]]; then
  echo "service api websocket prebuild failed" >&2
  exit "$prebuild_code"
fi
if [[ ! -x "$prebuilt_node_bin" ]]; then
  echo "expected isolated prebuilt kamn-node binary to be executable" >&2
  exit 1
fi

validation_output="$(
  KAMN_SERVICE_API_WEBSOCKET_SKIP_BUILD=1 \
    KAMN_SERVICE_API_WEBSOCKET_NODE_BIN="$prebuilt_node_bin" \
    bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected websocket live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected websocket live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_upgrade_status=verified$'; then
  echo "expected websocket upgrade status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_session_lifecycle_status=verified$'; then
  echo "expected websocket session lifecycle status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_heartbeat_timeout_status=verified$'; then
  echo "expected websocket heartbeat-timeout status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_idle_timeout_contract_status=verified$'; then
  echo "expected websocket idle-timeout contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected websocket fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^probe_status=verified$'; then
  echo "expected websocket probe status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_reason_registry_status=verified$'; then
  echo "expected websocket reason registry status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^protocol_session_docs_contract_status=verified$'; then
  echo "expected protocol/session docs contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1$'; then
  echo "expected protocol/session reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing$'; then
  echo "expected protocol/session reason codes csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_lifecycle_reason_taxonomy_version=kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1$'; then
  echo "expected websocket lifecycle reason taxonomy version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_lifecycle_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing$'; then
  echo "expected websocket lifecycle reason codes csv marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-validation.v1":
    raise SystemExit("unexpected websocket live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected websocket live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected websocket live validation final_decision=GO")
if payload.get("websocket_upgrade_status") != "verified":
    raise SystemExit("expected websocket_upgrade_status=verified")
if payload.get("websocket_session_lifecycle_status") != "verified":
    raise SystemExit("expected websocket_session_lifecycle_status=verified")
if payload.get("websocket_heartbeat_timeout_status") != "verified":
    raise SystemExit("expected websocket_heartbeat_timeout_status=verified")
if payload.get("websocket_idle_timeout_contract_status") != "verified":
    raise SystemExit("expected websocket_idle_timeout_contract_status=verified")
if payload.get("fail_closed_status") != "verified":
    raise SystemExit("expected fail_closed_status=verified")
if payload.get("probe_status") != "verified":
    raise SystemExit("expected probe_status=verified")
if payload.get("websocket_reason_registry_status") != "verified":
    raise SystemExit("expected websocket_reason_registry_status=verified")
if payload.get("protocol_session_docs_contract_status") != "verified":
    raise SystemExit("expected protocol_session_docs_contract_status=verified")
if payload.get("service_api_protocol_session_reason_taxonomy_version") != "kamn.runtime.service-api.protocol-session-reason-taxonomy.v1":
    raise SystemExit("expected service_api_protocol_session_reason_taxonomy_version marker")
if payload.get("service_api_protocol_session_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing":
    raise SystemExit("expected service_api_protocol_session_reason_codes_csv marker")
if payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected websocket_lifecycle_reason_taxonomy_version marker")
if payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected websocket_lifecycle_reason_codes_csv marker")
PY

echo "service api websocket live validation tests passed."
