#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/runtime/run_live_network_pilot_deep_lane.sh \
    [--event-name <schedule|workflow_dispatch>] \
    [--output-json <path>] \
    [--max-seconds <int>] \
    [--skip-suite]
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_LANE="$ROOT_DIR/scripts/runtime/run_live_network_smoke_lane.sh"
FAILOVER_SUITE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_suite.sh"
SUMMARY_GENERATOR="$ROOT_DIR/scripts/runtime/generate_live_network_pilot_artifact_summary.sh"
SUMMARY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh"

event_name="${GITHUB_EVENT_NAME:-schedule}"
output_json="$ROOT_DIR/live-network-pilot-report.json"
max_seconds="${KAMN_LIVE_NETWORK_PILOT_DEEP_MAX_SECONDS:-300}"
skip_suite=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --event-name)
      event_name="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    --skip-suite)
      skip_suite=true
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ ! "$max_seconds" =~ ^[0-9]+$ ]]; then
  fail "--max-seconds must be a non-negative integer"
fi

case "$event_name" in
  schedule|workflow_dispatch) ;;
  *)
    fail "scheduled/manual-only cadence policy requires event schedule or workflow_dispatch"
    ;;
esac

cadence="manual"
if [[ "$event_name" == "schedule" ]]; then
  cadence="scheduled"
fi

if [[ ! -x "$SMOKE_LANE" || ! -x "$FAILOVER_SUITE" || ! -x "$SUMMARY_GENERATOR" || ! -x "$SUMMARY_CHECKER" ]]; then
  fail "expected live-network deep lane dependencies to be executable"
fi

mkdir -p "$(dirname "$output_json")"

smoke_report="$(mktemp)"
failover_report="$(mktemp)"
trap 'rm -f "$smoke_report" "$failover_report"' EXIT

start_epoch="$(date +%s)"

set +e
bash "$SMOKE_LANE" --output-json "$smoke_report" >/dev/null
smoke_code=$?
set -e

if [[ ! -s "$smoke_report" ]]; then
  smoke_status="fail"
  smoke_decision="NO-GO"
  smoke_elapsed_seconds=0
else
  smoke_status="$(python3 - "$smoke_report" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload.get("status", "fail"))
PY
)"
  smoke_decision="$(python3 - "$smoke_report" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload.get("final_decision", "NO-GO"))
PY
)"
  smoke_elapsed_seconds="$(python3 - "$smoke_report" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload.get("elapsed_seconds", 0))
PY
)"
fi

suite_args=(--event-name schedule --output-json "$failover_report")
if [[ "$skip_suite" == true ]]; then
  suite_args+=(--skip-suite)
fi

set +e
bash "$FAILOVER_SUITE" "${suite_args[@]}" >/dev/null
failover_code=$?
set -e

if [[ ! -s "$failover_report" ]]; then
  failover_status="fail"
else
  failover_status="$(python3 - "$failover_report" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload.get("status", "fail"))
PY
)"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
budget_status="within"
if [[ "$elapsed_seconds" -gt "$max_seconds" ]]; then
  budget_status="exceeded"
fi

evidence_complete=true
if [[ ! -s "$smoke_report" || ! -s "$failover_report" ]]; then
  evidence_complete=false
fi

deep_status="pass"
deep_decision="GO"
if [[ "$smoke_code" -ne 0 || "$failover_code" -ne 0 || "$failover_status" != "pass" || "$budget_status" != "within" ]]; then
  deep_status="fail"
  deep_decision="NO-GO"
fi

summary_output="$(
  bash "$SUMMARY_GENERATOR" \
    --output-file "$output_json" \
    --event-name "$event_name" \
    --cadence "$cadence" \
    --smoke-status "$smoke_status" \
    --smoke-decision "$smoke_decision" \
    --smoke-elapsed-seconds "$smoke_elapsed_seconds" \
    --deep-status "$deep_status" \
    --deep-decision "$deep_decision" \
    --deep-elapsed-seconds "$elapsed_seconds" \
    --budget-status "$budget_status" \
    --evidence-complete "$evidence_complete" \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$summary_output" | grep -q '^status=generated$'; then
  fail "expected live-network pilot artifact summary generator to produce status=generated"
fi

check_output="$(bash "$SUMMARY_CHECKER" --summary-file "$output_json")"
final_decision="$(printf '%s\n' "$check_output" | awk -F= '/^final_decision=/{print $2; exit}')"
if [[ -z "$final_decision" ]]; then
  fail "live-network pilot summary checker did not emit final_decision"
fi

if [[ "$final_decision" != "GO" ]]; then
  fail "live-network pilot deep lane produced final_decision=${final_decision}"
fi

echo "live-network pilot deep lane tests passed."
