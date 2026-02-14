#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_reason_code_compatibility_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api reason-code compatibility policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/service-api-reason-code-compatibility-live-summary.json"
cat >"$report_file" <<'JSON'
{
  "schema_version": "kamn.runtime.service-api-reason-code-compatibility-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "reason_registry_status": "verified",
  "route_error_mapping_status": "verified",
  "replay_error_mapping_status": "verified",
  "websocket_error_mapping_status": "verified",
  "fail_closed_status": "verified",
  "performance_budget_status": "verified",
  "fail_closed_reason_code": "service_api_payload_structure_invalid",
  "elapsed_seconds": 4
}
JSON

policy_report="$TMP_DIR/service-api-reason-code-compatibility-live-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected service api reason-code compatibility policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api reason-code compatibility policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_reason_code_policy_status=verified$'; then
  echo "expected service api reason-code compatibility policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-reason-code-compatibility-live-policy-report.v1":
    raise SystemExit("unexpected service api reason-code compatibility policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("service_api_reason_code_policy_status") != "verified":
    raise SystemExit("expected service_api_reason_code_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
PY

tampered_report="$TMP_DIR/service-api-reason-code-compatibility-live-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["route_error_mapping_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-reason-code-compatibility-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered service api reason-code compatibility report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'service_api_reason_code_policy_marker_missing:route_error_mapping_status'; then
  echo "expected deterministic mismatch reason code for tampered reason-code policy validation" >&2
  exit 1
fi

echo "service api reason-code compatibility live policy checker tests passed."
