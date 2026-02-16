#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_metrics_scrape_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_metrics_scrape_live_policy.sh"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
ROADMAP_DOC="$ROOT_DIR/docs/plans/2026-02-08-production-service-roadmap.md"

output_json=""
policy_output_json=""
max_seconds="${KAMN_LOCAL_METRICS_SCRAPE_CONTRACT_MAX_SECONDS:-240}"
ci_fast_gate="PASS"
mode="dry-run"

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

summary_report="$TMP_DIR/local-metrics-scrape-live-summary.json"
policy_report="$TMP_DIR/local-metrics-scrape-live-policy.json"
tampered_report="$TMP_DIR/local-metrics-scrape-live-summary.tampered.json"

validation_output="$(
  bash "$VALIDATION_SCRIPT" \
    --mode "$mode" \
    --max-seconds "$max_seconds" \
    --output-json "$summary_report"
)"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected local metrics scrape validation status=pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected local metrics scrape validation final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q "^lane_mode=$mode$"; then
  echo "expected local metrics scrape validation lane mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^metrics_stream_readiness_status=verified$'; then
  echo "expected local metrics scrape validation metrics readiness marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^scrape_latency_budget_status=verified$'; then
  echo "expected local metrics scrape validation scrape latency budget marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^local_scrape_probe_status=verified$'; then
  echo "expected local metrics scrape validation local-scrape marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^prometheus_payload_status=verified$'; then
  echo "expected local metrics scrape validation prometheus marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^health_endpoint_status=verified$'; then
  echo "expected local metrics scrape validation health marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^fail_closed_status=verified$'; then
  echo "expected local metrics scrape validation fail-closed marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^metrics_emission_reason_taxonomy_version=kamn.runtime.metrics-emission-reason-taxonomy.v1$'; then
  echo "expected local metrics scrape validation reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^metrics_emission_reason_codes_csv=metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch$'; then
  echo "expected local metrics scrape validation reason taxonomy csv marker" >&2
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
  echo "expected local metrics scrape policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local metrics scrape policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_metrics_scrape_policy_status=verified$'; then
  echo "expected local metrics scrape policy checker status marker" >&2
  exit 1
fi

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["local_scrape_probe_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate "$ci_fast_gate" \
    --output-json "$TMP_DIR/local-metrics-scrape-live-policy.tampered.json" 2>&1
)"
tampered_policy_code=$?
set -e

if [ "$tampered_policy_code" -eq 0 ]; then
  echo "expected tampered local metrics scrape report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_policy_output" | grep -q 'local_metrics_scrape_policy_marker_missing:local_scrape_probe_status'; then
  echo "expected deterministic fail-closed reason for tampered local metrics scrape report" >&2
  exit 1
fi

for required_ref in \
  "validate_local_metrics_scrape_live.sh" \
  "check_local_metrics_scrape_live_policy.sh" \
  "validate_local_metrics_scrape_live_contract_lane.sh" \
  "test_validate_local_metrics_scrape_live_contract_lane.sh" \
  "test_check_local_metrics_scrape_live_policy.sh"; do
  if ! grep -q "$required_ref" "$STRATEGY_DOC"; then
    echo "expected CI strategy docs to reference $required_ref" >&2
    exit 1
  fi
done
if ! grep -q "local metrics scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode." "$STRATEGY_DOC"; then
  echo "expected CI strategy docs to include local metrics scrape run-mode exclusion marker" >&2
  exit 1
fi

if ! grep -q "Task #3271" "$ROADMAP_DOC"; then
  echo "expected roadmap marker for Task #3271" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/validate_local_metrics_scrape_live_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference local metrics scrape contract lane script" >&2
  exit 1
fi
if ! grep -q "scripts/runtime/check_local_metrics_scrape_live_policy.sh" "$ROADMAP_DOC"; then
  echo "expected roadmap to reference local metrics scrape policy checker script" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "local metrics scrape contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

lane_report="$TMP_DIR/local-metrics-scrape-live-contract-lane-report.json"
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

if summary_report.get("schema_version") != "kamn.runtime.local-metrics-scrape-live-report.v1":
    raise SystemExit("unexpected local metrics scrape live summary schema")
if policy_report.get("schema_version") != "kamn.runtime.local-metrics-scrape-live-policy-report.v1":
    raise SystemExit("unexpected local metrics scrape live policy schema")
if summary_report.get("final_decision") != "GO":
    raise SystemExit("expected local metrics scrape summary final_decision=GO")
if policy_report.get("final_decision") != "GO":
    raise SystemExit("expected local metrics scrape policy final_decision=GO")

lane_report = {
    "schema_version": "kamn.runtime.local-metrics-scrape-live-contract-lane-report.v1",
    "status": "pass",
    "final_decision": "GO",
    "lane_mode": mode,
    "metrics_stream_readiness_status": summary_report.get(
        "metrics_stream_readiness_status"
    ),
    "scrape_latency_budget_status": summary_report.get("scrape_latency_budget_status"),
    "metrics_emission_reason_taxonomy_version": summary_report.get(
        "metrics_emission_reason_taxonomy_version"
    ),
    "metrics_emission_reason_codes_csv": summary_report.get(
        "metrics_emission_reason_codes_csv"
    ),
    "local_metrics_scrape_contract_status": "verified",
    "local_metrics_scrape_policy_status": policy_report.get(
        "local_metrics_scrape_policy_status"
    ),
    "docs_contract_status": "verified",
    "fail_closed_status": "verified",
    "fail_closed_reason_code": "local_metrics_scrape_policy_marker_missing:local_scrape_probe_status",
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
echo "metrics_stream_readiness_status=verified"
echo "scrape_latency_budget_status=verified"
echo "metrics_emission_reason_taxonomy_version=kamn.runtime.metrics-emission-reason-taxonomy.v1"
echo "metrics_emission_reason_codes_csv=metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch"
echo "local_metrics_scrape_contract_status=verified"
echo "local_metrics_scrape_policy_status=verified"
echo "docs_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=local_metrics_scrape_policy_marker_missing:local_scrape_probe_status"
echo "performance_budget_status=verified"
if [[ -n "$output_json" ]]; then
  echo "contract_lane_report=$output_json"
fi
if [[ -n "$policy_output_json" ]]; then
  echo "policy_report=$policy_output_json"
fi
