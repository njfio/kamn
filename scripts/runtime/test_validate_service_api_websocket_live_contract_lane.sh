#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live_contract_lane.sh"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_websocket_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_websocket_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected websocket live contract lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected websocket live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected websocket live policy checker script to be executable" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-websocket-live-contract-lane-report.json"
policy_report="$TMP_DIR/service-api-websocket-live-policy-report.json"

lane_output="$(
  bash "$CONTRACT_LANE" \
    --output-json "$lane_report" \
    --policy-output-json "$policy_report"
)"
if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected websocket live contract lane status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected websocket live contract lane final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_contract_status=verified$'; then
  echo "expected websocket live contract lane status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^service_api_websocket_policy_status=verified$'; then
  echo "expected websocket live contract lane policy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_session_lifecycle_status=verified$'; then
  echo "expected websocket session lifecycle status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_heartbeat_timeout_status=verified$'; then
  echo "expected websocket heartbeat-timeout status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_idle_timeout_contract_status=verified$'; then
  echo "expected websocket idle-timeout contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_reason_registry_status=verified$'; then
  echo "expected websocket reason registry status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_lifecycle_reason_taxonomy_version=kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1$'; then
  echo "expected websocket lifecycle reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^websocket_lifecycle_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing$'; then
  echo "expected websocket lifecycle reason taxonomy csv marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^fail_closed_reason_code=service_api_websocket_policy_marker_missing:websocket_session_lifecycle_status$'; then
  echo "expected websocket live contract lane fail-closed reason marker" >&2
  exit 1
fi

python3 - "$lane_report" "$policy_report" <<'PY'
import json
import pathlib
import sys

lane_payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if lane_payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-contract-lane-report.v1":
    raise SystemExit("unexpected websocket live contract lane report schema")
if lane_payload.get("status") != "pass":
    raise SystemExit("expected websocket contract lane status=pass")
if lane_payload.get("final_decision") != "GO":
    raise SystemExit("expected websocket contract lane final_decision=GO")
if lane_payload.get("service_api_websocket_contract_status") != "verified":
    raise SystemExit("expected service_api_websocket_contract_status=verified")
if lane_payload.get("service_api_websocket_policy_status") != "verified":
    raise SystemExit("expected service_api_websocket_policy_status=verified")
if lane_payload.get("websocket_session_lifecycle_status") != "verified":
    raise SystemExit("expected websocket_session_lifecycle_status=verified")
if lane_payload.get("websocket_heartbeat_timeout_status") != "verified":
    raise SystemExit("expected websocket_heartbeat_timeout_status=verified")
if lane_payload.get("websocket_idle_timeout_contract_status") != "verified":
    raise SystemExit("expected websocket_idle_timeout_contract_status=verified")
if lane_payload.get("websocket_reason_registry_status") != "verified":
    raise SystemExit("expected websocket_reason_registry_status=verified")
if lane_payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_taxonomy_version marker")
if lane_payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_codes_csv marker")
if lane_payload.get("docs_contract_status") != "verified":
    raise SystemExit("expected docs_contract_status=verified")
if lane_payload.get("performance_budget_status") != "verified":
    raise SystemExit("expected performance_budget_status=verified")

policy_payload = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if policy_payload.get("schema_version") != "kamn.runtime.service-api-websocket-live-policy-report.v1":
    raise SystemExit("unexpected websocket live policy report schema")
if policy_payload.get("final_decision") != "GO":
    raise SystemExit("expected websocket policy final_decision=GO")
if policy_payload.get("service_api_websocket_policy_status") != "verified":
    raise SystemExit("expected service_api_websocket_policy_status=verified in policy report")
if policy_payload.get("websocket_lifecycle_reason_taxonomy_version") != "kamn.runtime.service-api-websocket-lifecycle-reason-taxonomy.v1":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_taxonomy_version marker in policy report")
if policy_payload.get("websocket_lifecycle_reason_codes_csv") != "service_api_ws_upgrade_header_missing,service_api_ws_version_header_invalid,service_api_auth_sender_did_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing":
    raise SystemExit("expected deterministic websocket_lifecycle_reason_codes_csv marker in policy report")
PY

set +e
invalid_ci_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate MAYBE 2>&1
)"
invalid_ci_fast_gate_code=$?
set -e
if [ "$invalid_ci_fast_gate_code" -eq 0 ]; then
  echo "expected websocket live contract lane to reject invalid ci-fast-gate value" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_ci_fast_gate_output" | grep -q 'ci-fast-gate must be PASS or FAIL'; then
  echo "expected deterministic invalid ci-fast-gate marker for websocket live contract lane" >&2
  exit 1
fi

set +e
blocked_fast_gate_output="$(
  bash "$CONTRACT_LANE" \
    --ci-fast-gate FAIL 2>&1
)"
blocked_fast_gate_code=$?
set -e
if [ "$blocked_fast_gate_code" -eq 0 ]; then
  echo "expected websocket live contract lane to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$blocked_fast_gate_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for websocket live contract lane" >&2
  exit 1
fi

echo "service api websocket live contract lane tests passed."
