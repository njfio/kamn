#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_structured_logging_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_structured_logging_live_policy.sh"
OBSERVABILITY_DOC="$ROOT_DIR/docs/foundation/observability-slo-dashboards.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
RUNTIME_LAYOUT_DOC="$ROOT_DIR/docs/architecture/runtime-layout.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_STRUCTURED_LOGGING_CONTRACT_MAX_SECONDS:-180}"
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

for required_doc in "$OBSERVABILITY_DOC" "$ROADMAP_DOC" "$STRATEGY_DOC" "$RUNTIME_LAYOUT_DOC"; do
  if [ ! -f "$required_doc" ]; then
    echo "expected required documentation file '$required_doc'" >&2
    exit 1
  fi
done

start_epoch="$(date +%s)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

summary_report="$TMP_DIR/structured-logging-live-summary.json"
policy_report="$TMP_DIR/structured-logging-live-policy.json"
tampered_report="$TMP_DIR/structured-logging-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --output-json "$summary_report" \
    --max-seconds "$max_seconds"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected structured logging live validation pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected structured logging live validation GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_id_parity_status=verified$'; then
  echo "expected structured logging live validation correlation parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^trace_classification_contract_status=verified$'; then
  echo "expected structured logging live validation trace-classification marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^log_classification_gate_status=verified$'; then
  echo "expected structured logging live validation log-classification gate marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1$'; then
  echo "expected structured logging live validation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^telemetry_schema_version=kamn.runtime.structured-logging-telemetry.v1$'; then
  echo "expected structured logging live validation telemetry schema version marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^telemetry_schema_contract_status=verified$'; then
  echo "expected structured logging live validation telemetry schema contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^telemetry_schema_reason_taxonomy_version=kamn.runtime.structured-logging-telemetry-schema-reason-taxonomy.v1$'; then
  echo "expected structured logging live validation telemetry schema reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^telemetry_schema_reason_codes_csv=structured_logging_telemetry_schema_version_mismatch,correlation_id_parity_bypass_detected$'; then
  echo "expected structured logging live validation telemetry schema reason codes marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_error_reason_taxonomy_version=kamn.runtime.correlation-error-reason-taxonomy.v1$'; then
  echo "expected structured logging live validation correlation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_error_reason_codes_csv=correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped$'; then
  echo "expected structured logging live validation correlation reason taxonomy csv marker" >&2
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
  echo "expected structured logging policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected structured logging policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^structured_logging_policy_status=verified$'; then
  echo "expected structured logging policy checker status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("structured_logging_contract_status", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/structured-logging-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e
if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered structured logging summary report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'structured_logging_policy_marker_missing:structured_logging_contract_status'; then
  echo "expected deterministic fail-closed reason for tampered structured logging report" >&2
  exit 1
fi

for required_ref in \
  "check_structured_logging_live_policy.sh" \
  "test_check_structured_logging_live_policy.sh" \
  "validate_structured_logging_live_contract_lane.sh" \
  "test_validate_structured_logging_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$OBSERVABILITY_DOC"; then
    echo "expected observability docs to reference $required_ref" >&2
    exit 1
  fi
done

for required_ref in \
  "scripts/runtime/check_structured_logging_live_policy.sh" \
  "scripts/runtime/validate_structured_logging_live_contract_lane.sh" \
  "Task #4641, Subtasks #4645 and #4646"; do
  if ! grep -q "$required_ref" "$ROADMAP_DOC"; then
    echo "expected roadmap docs to reference $required_ref" >&2
    exit 1
  fi
done

for required_ref in \
  "check_structured_logging_live_policy.sh" \
  "validate_structured_logging_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done

for required_ref in \
  "structured_logging_policy_status" \
  "structured_logging_contract_lane_status" \
  "reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1"; do
  if ! grep -q "$required_ref" "$RUNTIME_LAYOUT_DOC"; then
    echo "expected runtime layout docs to reference $required_ref" >&2
    exit 1
  fi
done

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "structured logging contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/structured-logging-live-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])

if summary_report.get("schema_version") != "kamn.runtime.structured-logging-live-validation.v1":
    raise SystemExit("unexpected structured logging live summary schema")
if policy_report.get("schema_version") != "kamn.runtime.structured-logging-live-policy-report.v1":
    raise SystemExit("unexpected structured logging live policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected structured logging live summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected structured logging live policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.structured-logging-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "structured_logging_contract_status": summary_report.get(
        "structured_logging_contract_status"
    ),
    "telemetry_schema_version": summary_report.get("telemetry_schema_version"),
    "telemetry_schema_contract_status": summary_report.get(
        "telemetry_schema_contract_status"
    ),
    "correlation_contract_status": summary_report.get("correlation_contract_status"),
    "structured_logging_policy_status": policy_report.get(
        "structured_logging_policy_status"
    ),
    "structured_logging_contract_lane_status": "verified",
    "correlation_id_parity_status": summary_report.get("correlation_id_parity_status"),
    "trace_classification_contract_status": summary_report.get(
        "trace_classification_contract_status"
    ),
    "log_classification_gate_status": summary_report.get(
        "log_classification_gate_status"
    ),
    "reason_taxonomy_version": summary_report.get("reason_taxonomy_version"),
    "telemetry_schema_reason_taxonomy_version": summary_report.get(
        "telemetry_schema_reason_taxonomy_version"
    ),
    "telemetry_schema_reason_codes_csv": summary_report.get(
        "telemetry_schema_reason_codes_csv"
    ),
    "correlation_error_reason_taxonomy_version": summary_report.get(
        "correlation_error_reason_taxonomy_version"
    ),
    "correlation_error_reason_codes_csv": summary_report.get(
        "correlation_error_reason_codes_csv"
    ),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "structured_logging_policy_marker_missing:structured_logging_contract_status",
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
echo "structured_logging_contract_status=verified"
echo "telemetry_schema_version=kamn.runtime.structured-logging-telemetry.v1"
echo "telemetry_schema_contract_status=verified"
echo "correlation_contract_status=verified"
echo "structured_logging_policy_status=verified"
echo "structured_logging_contract_lane_status=verified"
echo "correlation_id_parity_status=verified"
echo "trace_classification_contract_status=verified"
echo "log_classification_gate_status=verified"
echo "reason_taxonomy_version=kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1"
echo "telemetry_schema_reason_taxonomy_version=kamn.runtime.structured-logging-telemetry-schema-reason-taxonomy.v1"
echo "telemetry_schema_reason_codes_csv=structured_logging_telemetry_schema_version_mismatch,correlation_id_parity_bypass_detected"
echo "correlation_error_reason_taxonomy_version=kamn.runtime.correlation-error-reason-taxonomy.v1"
echo "correlation_error_reason_codes_csv=correlation_id_missing,correlation_id_mismatch,trace_classification_unmapped"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=structured_logging_policy_marker_missing:structured_logging_contract_status"
echo "performance_budget_status=verified"
