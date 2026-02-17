#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_retry_diagnostics_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_LOCAL_RETRY_DIAGNOSTICS_CONTRACT_MAX_SECONDS:-180}"
ci_fast_gate="PASS"
mode="dry-run"
nonce_retry_script=""
structured_logging_script=""
EXPECTED_REASON_TAXONOMY_VERSION="kamn.runtime.local-retry-diagnostics-reason-taxonomy.v2"
EXPECTED_REASON_CODES_CSV="local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,local_retry_envelope_exhaustion_fail_closed_missing,local_retry_reconnect_attempt_bound_drift,local_retry_reconnect_backoff_bound_drift,ci_local_network_budget_boundary_exceeded"

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
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    --nonce-retry-script)
      nonce_retry_script="${2:-}"
      shift 2
      ;;
    --structured-logging-script)
      structured_logging_script="${2:-}"
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
if [ "$max_seconds" -gt 240 ]; then
  echo "max-seconds must be <= 240 for ci-local contract lane" >&2
  exit 1
fi
if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi
if [[ "$mode" != "dry-run" && "$mode" != "run" ]]; then
  echo "mode must be dry-run or run" >&2
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

summary_report="$TMP_DIR/runtime-local-retry-diagnostics-summary.json"
policy_report="$TMP_DIR/runtime-local-retry-diagnostics-policy.json"
tampered_report="$TMP_DIR/runtime-local-retry-diagnostics-summary.tampered.json"

validation_command=(
  bash "$VALIDATION_SCRIPT"
  --mode "$mode"
  --max-seconds "$max_seconds"
  --output-json "$summary_report"
)
if [[ -n "$nonce_retry_script" ]]; then
  validation_command+=(--nonce-retry-script "$nonce_retry_script")
fi
if [[ -n "$structured_logging_script" ]]; then
  validation_command+=(--structured-logging-script "$structured_logging_script")
fi

validation_output="$("${validation_command[@]}")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local retry/diagnostics live validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local retry/diagnostics live validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^retry_contract_status=verified$'; then
  echo "expected local retry/diagnostics live validation retry marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^correlation_diagnostics_status=verified$'; then
  echo "expected local retry/diagnostics live validation correlation marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^retry_readiness_status=verified$'; then
  echo "expected local retry/diagnostics live validation retry readiness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^retry_jitter_parity_status=verified$'; then
  echo "expected local retry/diagnostics live validation retry jitter parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^retry_envelope_exhaustion_fail_closed_status=verified$'; then
  echo "expected local retry/diagnostics live validation retry envelope exhaustion fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconnect_attempt_bound_status=verified$'; then
  echo "expected local retry/diagnostics live validation reconnect-attempt bound marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^reconnect_backoff_bound_status=verified$'; then
  echo "expected local retry/diagnostics live validation reconnect-backoff bound marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^retry_envelope_max_attempts=3$'; then
  echo "expected local retry/diagnostics live validation retry envelope max-attempts marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^retry_envelope_max_backoff_seconds=8$'; then
  echo "expected local retry/diagnostics live validation retry envelope max-backoff marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$"; then
  echo "expected local retry/diagnostics live validation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^reason_codes_csv=$EXPECTED_REASON_CODES_CSV$"; then
  echo "expected local retry/diagnostics live validation reason codes taxonomy marker" >&2
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
  echo "expected local retry/diagnostics policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local retry/diagnostics policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_retry_diagnostics_policy_status=verified$'; then
  echo "expected local retry/diagnostics policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION$"; then
  echo "expected local retry/diagnostics policy checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_codes_csv=$EXPECTED_REASON_CODES_CSV$"; then
  echo "expected local retry/diagnostics policy checker reason codes taxonomy marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["correlation_diagnostics_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/runtime-local-retry-diagnostics-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e

if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered local retry/diagnostics report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status'; then
  echo "expected deterministic fail-closed reason for tampered local retry/diagnostics report" >&2
  exit 1
fi

for required_ref in \
  "validate_local_retry_diagnostics_live.sh" \
  "check_local_retry_diagnostics_live_policy.sh" \
  "validate_local_retry_diagnostics_live_contract_lane.sh" \
  "test_validate_local_retry_diagnostics_live.sh" \
  "test_check_local_retry_diagnostics_live_policy.sh" \
  "test_validate_local_retry_diagnostics_live_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "local retry/diagnostics run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include local retry/diagnostics run-mode exclusion marker" >&2
  exit 1
fi

if ! grep -q "Task #3216, Subtask #3217" "$ROADMAP_DOC"; then
  echo "expected roadmap marker for Task #3216, Subtask #3217" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_local_retry_diagnostics_live_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference local retry/diagnostics contract lane script" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/check_local_retry_diagnostics_live_policy.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference local retry/diagnostics policy checker script" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "local retry/diagnostics contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/runtime-local-retry-diagnostics-contract-lane-report.json"
python3 - "$summary_report" "$policy_report" "$lane_report" "$elapsed_seconds" "$max_seconds" "$mode" <<'PY'
import json
import pathlib
import sys

summary_report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy_report = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
lane_report_file = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])
mode = sys.argv[6]

if summary_report.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-report.v1":
    raise SystemExit("unexpected local retry/diagnostics live summary schema")
if policy_report.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-policy-report.v1":
    raise SystemExit("unexpected local retry/diagnostics live policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected local retry/diagnostics summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected local retry/diagnostics policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.local-retry-diagnostics-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "local_retry_diagnostics_contract_status": "verified",
    "local_retry_diagnostics_policy_status": policy_report.get(
        "local_retry_diagnostics_policy_status"
    ),
    "retry_readiness_status": summary_report.get("retry_readiness_status"),
    "retry_jitter_parity_status": summary_report.get("retry_jitter_parity_status"),
    "retry_envelope_exhaustion_fail_closed_status": summary_report.get(
        "retry_envelope_exhaustion_fail_closed_status"
    ),
    "reconnect_attempt_bound_status": summary_report.get("reconnect_attempt_bound_status"),
    "reconnect_backoff_bound_status": summary_report.get("reconnect_backoff_bound_status"),
    "retry_envelope_max_attempts": summary_report.get("retry_envelope_max_attempts"),
    "retry_envelope_max_backoff_seconds": summary_report.get(
        "retry_envelope_max_backoff_seconds"
    ),
    "reason_taxonomy_version": summary_report.get("reason_taxonomy_version"),
    "reason_codes_csv": summary_report.get("reason_codes_csv"),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status",
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
echo "lane_mode=$mode"
echo "local_retry_diagnostics_contract_status=verified"
echo "local_retry_diagnostics_policy_status=verified"
echo "retry_readiness_status=verified"
echo "retry_jitter_parity_status=verified"
echo "retry_envelope_exhaustion_fail_closed_status=verified"
echo "reconnect_attempt_bound_status=verified"
echo "reconnect_backoff_bound_status=verified"
echo "retry_envelope_max_attempts=3"
echo "retry_envelope_max_backoff_seconds=8"
echo "reason_taxonomy_version=$EXPECTED_REASON_TAXONOMY_VERSION"
echo "reason_codes_csv=$EXPECTED_REASON_CODES_CSV"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status"
echo "performance_budget_status=verified"
