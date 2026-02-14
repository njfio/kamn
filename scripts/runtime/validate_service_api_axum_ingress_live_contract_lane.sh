#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_service_api_axum_ingress_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_service_api_axum_ingress_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_SERVICE_API_AXUM_INGRESS_CONTRACT_MAX_SECONDS:-180}"
ci_fast_gate="PASS"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --policy-output-json)
      policy_output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi

for required_exec in "$VALIDATION_SCRIPT" "$POLICY_CHECKER"; do
  if [ ! -x "$required_exec" ]; then
    echo "expected required executable script '$required_exec'" >&2
    exit 1
  fi
done
for required_doc in "$STRATEGY_DOC" "$ROADMAP_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/service-api-axum-ingress-live-summary.json"
policy_report="$TMP_DIR/service-api-axum-ingress-live-policy.json"
tampered_report="$TMP_DIR/service-api-axum-ingress-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --output-json "$summary_report" \
    --max-seconds "$max_seconds"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected service api axum ingress live validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api axum ingress live validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^keep_alive_status=verified$'; then
  echo "expected service api axum ingress keep-alive marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^body_size_guard_status=verified$'; then
  echo "expected service api axum ingress body-size marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^concurrency_status=verified$'; then
  echo "expected service api axum ingress concurrency marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^websocket_status=verified$'; then
  echo "expected service api axum ingress websocket marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^ingress_limit_config_status=verified$'; then
  echo "expected service api axum ingress config-matrix marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^docs_ingress_limit_matrix_status=verified$'; then
  echo "expected service api axum ingress docs parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -Eq '^api_max_requests_default=[1-9][0-9]*$'; then
  echo "expected service api axum ingress max-requests default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -Eq '^api_idle_timeout_default_ms=[1-9][0-9]*$'; then
  echo "expected service api axum ingress idle-timeout default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -Eq '^body_size_limit_bytes=[1-9][0-9]*$'; then
  echo "expected service api axum ingress body-size limit marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -Eq '^api_concurrency_limit_default=[1-9][0-9]*$'; then
  echo "expected service api axum ingress concurrency-limit default marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -Eq '^api_rate_limit_per_second_default=[1-9][0-9]*$'; then
  echo "expected service api axum ingress rate-limit default marker" >&2
  exit 1
fi

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected service api axum ingress policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected service api axum ingress policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^service_api_axum_ingress_policy_status=verified$'; then
  echo "expected service api axum ingress policy checker status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["concurrency_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/service-api-axum-ingress-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e

if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered service api axum ingress report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'service_api_axum_policy_marker_missing:concurrency_status'; then
  echo "expected deterministic fail-closed reason for tampered service api axum ingress report" >&2
  exit 1
fi

for required_ref in \
  "validate_service_api_axum_ingress_live.sh" \
  "check_service_api_axum_ingress_live_policy.sh" \
  "validate_service_api_axum_ingress_live_contract_lane.sh" \
  "test_validate_service_api_axum_ingress_live_contract_lane.sh" \
  "test_check_service_api_axum_ingress_live_policy.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "service api axum ingress run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include service api axum ingress run-mode exclusion marker" >&2
  exit 1
fi
if ! grep -q "ingress limit config matrix defaults remain parity-checked against source constants and API docs" "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include ingress-limit config matrix parity marker" >&2
  exit 1
fi

if ! grep -q "Task #3308" "$ROADMAP_DOC"; then
  echo "expected roadmap marker for Task #3308" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_service_api_axum_ingress_live_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference service api axum ingress contract lane script" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/check_service_api_axum_ingress_live_policy.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference service api axum ingress policy checker script" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "service api axum ingress contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/service-api-axum-ingress-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])

if summary_report.get("schema_version") != "kamn.runtime.service-api-axum-ingress-live-validation.v1":
    raise SystemExit("unexpected service api axum ingress live summary schema")
if policy_report.get("schema_version") != "kamn.runtime.service-api-axum-ingress-live-policy-report.v1":
    raise SystemExit("unexpected service api axum ingress live policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected service api axum ingress summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected service api axum ingress policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.service-api-axum-ingress-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "service_api_axum_ingress_contract_status": "verified",
    "service_api_axum_ingress_policy_status": policy_report.get(
        "service_api_axum_ingress_policy_status"
    ),
    "ingress_limit_config_status": summary_report.get("ingress_limit_config_status"),
    "docs_ingress_limit_matrix_status": summary_report.get(
        "docs_ingress_limit_matrix_status"
    ),
    "api_max_requests_default": summary_report.get("api_max_requests_default"),
    "api_idle_timeout_default_ms": summary_report.get("api_idle_timeout_default_ms"),
    "body_size_limit_bytes": summary_report.get("body_size_limit_bytes"),
    "api_concurrency_limit_default": summary_report.get("api_concurrency_limit_default"),
    "api_rate_limit_per_second_default": summary_report.get(
        "api_rate_limit_per_second_default"
    ),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "service_api_axum_policy_marker_missing:concurrency_status",
    "performance_budget_status": "verified",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
}
lane_report_file.write_text(json.dumps(lane_report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [[ -n "$output_json" ]]; then
  cp "$lane_report" "$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  cp "$policy_report" "$policy_output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "service_api_axum_ingress_contract_status=verified"
echo "service_api_axum_ingress_policy_status=verified"
echo "ingress_limit_config_status=verified"
echo "docs_ingress_limit_matrix_status=verified"
echo "api_max_requests_default=$(python3 - "$summary_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("api_max_requests_default", 0))
PY
)"
echo "api_idle_timeout_default_ms=$(python3 - "$summary_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("api_idle_timeout_default_ms", 0))
PY
)"
echo "body_size_limit_bytes=$(python3 - "$summary_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("body_size_limit_bytes", 0))
PY
)"
echo "api_concurrency_limit_default=$(python3 - "$summary_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("api_concurrency_limit_default", 0))
PY
)"
echo "api_rate_limit_per_second_default=$(python3 - "$summary_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("api_rate_limit_per_second_default", 0))
PY
)"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=service_api_axum_policy_marker_missing:concurrency_status"
echo "performance_budget_status=verified"
