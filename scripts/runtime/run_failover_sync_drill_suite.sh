#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SELECTOR="$ROOT_DIR/scripts/runtime/select_failover_sync_drill_lane.sh"
PREFLIGHT_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_preflight_contract_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_failover_sync_drill_deep_lane.sh"

event_name="${GITHUB_EVENT_NAME:-pull_request}"
output_json="$ROOT_DIR/failover-sync-drill-suite-report.json"
skip_suite=false
max_seconds=15
simulate_delay_seconds=0

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
    --simulate-delay-seconds)
      simulate_delay_seconds="${2:-}"
      shift 2
      ;;
    --skip-suite)
      skip_suite=true
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage:
  bash scripts/runtime/run_failover_sync_drill_suite.sh \
    [--event-name <github-event>] \
    [--output-json <path>] \
    [--max-seconds <preflight-budget>] \
    [--simulate-delay-seconds <seconds>] \
    [--skip-suite]
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ ! -x "$SELECTOR" ] || [ ! -x "$PREFLIGHT_LANE" ] || [ ! -x "$DEEP_LANE" ]; then
  echo "expected selector and failover/sync lane scripts to be executable" >&2
  exit 1
fi

mkdir -p "$(dirname "$output_json")"

# In GitHub Actions, GITHUB_OUTPUT is set for the parent step. Force stdout mode so
# selector key/value lines can be captured reliably inside this script.
selection="$(env -u GITHUB_OUTPUT bash "$SELECTOR" --event-name "$event_name")"
selected_lane="$(printf '%s\n' "$selection" | awk -F= '/^lane=/{print $2}')"
cadence="$(printf '%s\n' "$selection" | awk -F= '/^cadence=/{print $2}')"

if [ -z "$selected_lane" ]; then
  echo "selector did not produce a lane" >&2
  exit 1
fi

lane_report="$(mktemp)"
trap 'rm -f "$lane_report"' EXIT

if [ "$selected_lane" = "preflight" ]; then
  bash "$PREFLIGHT_LANE" \
    --output-json "$lane_report" \
    --max-seconds "$max_seconds" \
    --simulate-delay-seconds "$simulate_delay_seconds" \
    $([ "$skip_suite" = true ] && printf '%s' '--skip-suite')
elif [ "$selected_lane" = "deep" ]; then
  KAMN_FAILOVER_SYNC_DEEP_CADENCE=scheduled \
    bash "$DEEP_LANE" \
      --output-json "$lane_report" \
      $([ "$skip_suite" = true ] && printf '%s' '--skip-suite')
else
  echo "unsupported selected lane: $selected_lane" >&2
  exit 1
fi

lane_status="$(python3 - "$lane_report" <<'PY'
import json
import pathlib
import sys
payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
print(payload.get("status", "unknown"))
PY
)"

python3 - "$output_json" "$event_name" "$selected_lane" "$cadence" "$lane_status" "$lane_report" <<'PY'
import json
import pathlib
import sys

output_json, event_name, lane, cadence, status, lane_report = sys.argv[1:]

payload = {
    "schema_version": "kamn.runtime.failover-sync-drill-suite-report.v1",
    "event_name": event_name,
    "selected_lane": lane,
    "cadence": cadence,
    "status": status,
    "lane_report": json.loads(pathlib.Path(lane_report).read_text()),
}

pathlib.Path(output_json).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
PY

if [ "$lane_status" != "pass" ]; then
  echo "failover/sync drill suite failed in lane: $selected_lane" >&2
  exit 1
fi

echo "failover/sync drill suite tests passed."
