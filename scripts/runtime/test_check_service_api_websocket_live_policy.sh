#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected websocket live policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/service-api-websocket-live-summary.json"
cat >"$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.service-api-websocket-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "websocket_upgrade_status": "verified",
  "websocket_session_lifecycle_status": "verified",
  "websocket_heartbeat_timeout_status": "verified",
  "websocket_idle_timeout_contract_status": "verified",
  "fail_closed_status": "verified",
  "probe_status": "verified",
  "websocket_reason_registry_status": "verified",
  "websocket_lifecycle_reason_taxonomy_version": "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1",
  "websocket_lifecycle_reason_codes_csv": "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing",
  "api_idle_timeout_default_ms": 5000,
  "elapsed_seconds": 3
}
JSON

policy_report="$TMP_DIR/service-api-websocket-live-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected websocket live policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected websocket live policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_websocket_policy_status=verified$'; then
  echo "expected websocket live policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected websocket live policy checker normalized reason_codes_value marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-policy-report.v1":
    raise SystemExit("unexpected websocket live policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("service_api_websocket_policy_status") != "verified":
    raise SystemExit("expected service_api_websocket_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected policy checker success normalized reason_codes_value marker")
if payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_taxonomy_version marker")
if payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_codes_csv marker")
PY

tampered_lifecycle_report="$TMP_DIR/service-api-websocket-live-summary.lifecycle.tampered.json"
cp "$report_file" "$tampered_lifecycle_report"
python3 - "$tampered_lifecycle_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["websocket_session_lifecycle_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_lifecycle_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_lifecycle_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-websocket-live-policy.lifecycle.tampered.json" 2>&1
)"
tampered_lifecycle_code=$?
set -e

if [ "$tampered_lifecycle_code" -eq 0 ]; then
  echo "expected tampered websocket session lifecycle status to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_lifecycle_output" | grep -q 'service_api_websocket_policy_marker_missing:websocket_session_lifecycle_status'; then
  echo "expected deterministic mismatch reason code for websocket session lifecycle status tamper" >&2
  exit 1
fi

tampered_taxonomy_report="$TMP_DIR/service-api-websocket-live-summary.taxonomy.tampered.json"
cp "$report_file" "$tampered_taxonomy_report"
python3 - "$tampered_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["websocket_lifecycle_reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-websocket-live-policy.taxonomy.tampered.json" 2>&1
)"
tampered_taxonomy_code=$?
set -e

if [ "$tampered_taxonomy_code" -eq 0 ]; then
  echo "expected tampered websocket lifecycle taxonomy to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q 'service_api_websocket_policy_websocket_lifecycle_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic mismatch reason code for websocket lifecycle taxonomy tamper" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q '^reason_codes_value=service_api_websocket_policy_websocket_lifecycle_reason_taxonomy_version_mismatch$'; then
  echo "expected normalized reason_codes_value marker for websocket lifecycle taxonomy tamper" >&2
  exit 1
fi

missing_required_field_report="$TMP_DIR/service-api-websocket-live-summary.required-field-missing.json"
cp "$report_file" "$missing_required_field_report"
python3 - "$missing_required_field_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("websocket_lifecycle_reason_codes_csv", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
missing_required_field_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$missing_required_field_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-websocket-live-policy.required-field-missing.json" 2>&1
)"
missing_required_field_code=$?
set -e

if [ "$missing_required_field_code" -eq 0 ]; then
  echo "expected missing required websocket field tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_required_field_output" | grep -q 'service_api_websocket_policy_required_field_missing:websocket_lifecycle_reason_codes_csv'; then
  echo "expected deterministic required-field reason code for missing websocket_lifecycle_reason_codes_csv" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_required_field_output" | grep -q '^reason_codes_value=.*service_api_websocket_policy_required_field_missing:websocket_lifecycle_reason_codes_csv'; then
  echo "expected normalized reason_codes_value output to include missing required websocket field reason code" >&2
  exit 1
fi

echo "service api websocket live policy checker tests passed."
