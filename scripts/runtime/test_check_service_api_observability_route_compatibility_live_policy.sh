#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_observability_route_compatibility_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_observability_route_compatibility_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected service api observability route compatibility validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected service api observability route compatibility policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/service-api-observability-route-compatibility-summary.json"
bash "$VALIDATION_SCRIPT" \
  --mode dry-run \
  --output-json "$report_file" >/dev/null

policy_report="$TMP_DIR/service-api-observability-route-compatibility-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
for marker in \
  '^status=ok$' \
  '^final_decision=GO$' \
  '^service_api_observability_route_compatibility_policy_status=verified$' \
  '^reason_codes=none$' \
  '^reason_codes_value=none$' \
  '^reason_taxonomy_version=kamn.runtime.service-api-observability-route-compatibility-policy-reason-taxonomy.v1$' \
  '^reason_codes_csv=ci_fast_gate_failed,service_api_observability_route_compatibility_policy_command_count_invalid,service_api_observability_route_compatibility_policy_command_count_mismatch,service_api_observability_route_compatibility_policy_elapsed_seconds_invalid,service_api_observability_route_compatibility_policy_execution_reason_code_mismatch,service_api_observability_route_compatibility_policy_final_decision_invalid,service_api_observability_route_compatibility_policy_final_decision_mismatch,service_api_observability_route_compatibility_policy_lane_mode_invalid,service_api_observability_route_compatibility_policy_marker_missing,service_api_observability_route_compatibility_policy_matrix_row_compatibility_marker_missing,service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch,service_api_observability_route_compatibility_policy_matrix_row_count_mismatch,service_api_observability_route_compatibility_policy_matrix_row_duplicate,service_api_observability_route_compatibility_policy_matrix_row_id_invalid,service_api_observability_route_compatibility_policy_matrix_row_invalid,service_api_observability_route_compatibility_policy_matrix_row_method_mismatch,service_api_observability_route_compatibility_policy_matrix_row_missing,service_api_observability_route_compatibility_policy_matrix_row_route_mismatch,service_api_observability_route_compatibility_policy_matrix_row_status_mismatch,service_api_observability_route_compatibility_policy_matrix_row_surface_mismatch,service_api_observability_route_compatibility_policy_matrix_rows_invalid,service_api_observability_route_compatibility_policy_matrix_schema_mismatch,service_api_observability_route_compatibility_policy_schema_mismatch,service_api_observability_route_compatibility_policy_status_invalid$'; do
  if ! printf '%s\n' "$policy_output" | grep -q "$marker"; then
    echo "expected service api observability route compatibility policy marker: $marker" >&2
    exit 1
  fi
done

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.service-api-observability-route-compatibility-live-policy-report.v1":
    raise SystemExit("unexpected policy schema for service api observability route compatibility")
if payload.get("status") != "pass":
    raise SystemExit("expected policy status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if payload.get("service_api_observability_route_compatibility_policy_status") != "verified":
    raise SystemExit("expected compatibility policy status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.service-api-observability-route-compatibility-policy-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason taxonomy marker in compatibility policy report")
if payload.get("reason_codes_csv") != "ci_fast_gate_failed,service_api_observability_route_compatibility_policy_command_count_invalid,service_api_observability_route_compatibility_policy_command_count_mismatch,service_api_observability_route_compatibility_policy_elapsed_seconds_invalid,service_api_observability_route_compatibility_policy_execution_reason_code_mismatch,service_api_observability_route_compatibility_policy_final_decision_invalid,service_api_observability_route_compatibility_policy_final_decision_mismatch,service_api_observability_route_compatibility_policy_lane_mode_invalid,service_api_observability_route_compatibility_policy_marker_missing,service_api_observability_route_compatibility_policy_matrix_row_compatibility_marker_missing,service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch,service_api_observability_route_compatibility_policy_matrix_row_count_mismatch,service_api_observability_route_compatibility_policy_matrix_row_duplicate,service_api_observability_route_compatibility_policy_matrix_row_id_invalid,service_api_observability_route_compatibility_policy_matrix_row_invalid,service_api_observability_route_compatibility_policy_matrix_row_method_mismatch,service_api_observability_route_compatibility_policy_matrix_row_missing,service_api_observability_route_compatibility_policy_matrix_row_route_mismatch,service_api_observability_route_compatibility_policy_matrix_row_status_mismatch,service_api_observability_route_compatibility_policy_matrix_row_surface_mismatch,service_api_observability_route_compatibility_policy_matrix_rows_invalid,service_api_observability_route_compatibility_policy_matrix_schema_mismatch,service_api_observability_route_compatibility_policy_schema_mismatch,service_api_observability_route_compatibility_policy_status_invalid":
    raise SystemExit("expected deterministic reason code taxonomy marker in compatibility policy report")
if payload.get("reason_codes_value") != "none":
    raise SystemExit("expected reason_codes_value=none in compatibility policy report")
PY

tampered_report="$TMP_DIR/service-api-observability-route-compatibility-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
for row in payload.get("matrix_rows", []):
    if row.get("row_id") == "api_healthz_get":
        row["expected_status"] = 500
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/service-api-observability-route-compatibility-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered service api observability route compatibility report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:api_healthz_get'; then
  echo "expected deterministic mismatch reason code for tampered service api observability route compatibility policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_taxonomy_version=kamn.runtime.service-api-observability-route-compatibility-policy-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker for tampered service api observability route compatibility policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q '^reason_codes_value=service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:api_healthz_get$'; then
  echo "expected deterministic normalized reason_codes_value marker for tampered service api observability route compatibility policy validation" >&2
  exit 1
fi

echo "service api observability route compatibility live policy checker tests passed."
