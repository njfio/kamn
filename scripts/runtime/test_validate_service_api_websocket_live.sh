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

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
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
if payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected websocket_lifecycle_reason_taxonomy_version marker")
if payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected websocket_lifecycle_reason_codes_csv marker")
PY

echo "service api websocket live validation tests passed."
