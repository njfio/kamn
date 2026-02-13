#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SELECTOR="$ROOT_DIR/scripts/runtime/select_live_network_partition_reconnect_lane.sh"
SMOKE_LANE="$ROOT_DIR/scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_live_network_partition_reconnect_deep_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_network_partition_reconnect_policy.sh"
FIXTURE_FILE="$ROOT_DIR/fixtures/runtime/live_network_partition_reconnect_matrix_cases.json"
LIVE_NETWORK_DOC="$ROOT_DIR/docs/planning/live-network-wave.md"
GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"

event_name="${GITHUB_EVENT_NAME:-pull_request}"
output_json="$ROOT_DIR/live-network-partition-reconnect-contract-report.json"
max_seconds=""
simulate_delay_seconds=0
max_artifact_age_seconds="${KAMN_LIVE_NETWORK_PARTITION_RECONNECT_MAX_ARTIFACT_AGE_SECONDS:-900}"
fail_scenarios=""
ci_fast_gate="PASS"

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
    --max-artifact-age-seconds)
      max_artifact_age_seconds="${2:-}"
      shift 2
      ;;
    --fail-scenarios)
      fail_scenarios="${2:-}"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage:
  bash scripts/runtime/run_live_network_partition_reconnect_contract_lane.sh \
    [--event-name <github-event>] \
    [--output-json <path>] \
    [--max-seconds <seconds>] \
    [--simulate-delay-seconds <seconds>] \
    [--max-artifact-age-seconds <seconds>] \
    [--fail-scenarios <comma-separated-scenarios>] \
    [--ci-fast-gate PASS|FAIL]
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

for numeric_arg in "$simulate_delay_seconds" "$max_artifact_age_seconds"; do
  case "$numeric_arg" in
    ''|*[!0-9]*)
      echo "numeric arguments must be non-negative integers" >&2
      exit 1
      ;;
  esac
done

if [[ -n "$max_seconds" ]]; then
  case "$max_seconds" in
    ''|*[!0-9]*)
      echo "--max-seconds must be a non-negative integer" >&2
      exit 1
      ;;
  esac
fi

if [[ "$ci_fast_gate" != "PASS" && "$ci_fast_gate" != "FAIL" ]]; then
  echo "--ci-fast-gate must be PASS or FAIL" >&2
  exit 1
fi

if [[ ! -x "$SELECTOR" || ! -x "$SMOKE_LANE" || ! -x "$DEEP_LANE" || ! -x "$POLICY_CHECKER" ]]; then
  echo "expected partition/reconnect selector and lane scripts to be executable" >&2
  exit 1
fi

if [[ ! -f "$FIXTURE_FILE" ]]; then
  echo "expected partition/reconnect matrix fixture file to exist" >&2
  exit 1
fi

if [[ ! -f "$LIVE_NETWORK_DOC" || ! -f "$GONOGO_DOC" ]]; then
  echo "expected live-network planning and release go/no-go docs to exist" >&2
  exit 1
fi

selection_output="$(bash "$SELECTOR" --event-name "$event_name")"
selected_lane="$(printf '%s\n' "$selection_output" | awk -F= '/^lane=/{print $2}')"
cadence="$(printf '%s\n' "$selection_output" | awk -F= '/^cadence=/{print $2}')"

if [[ -z "$selected_lane" ]]; then
  echo "selector did not produce a lane" >&2
  exit 1
fi

if [[ "$selected_lane" == "smoke" ]]; then
  lane_command=(
    bash "$SMOKE_LANE"
    --event-name "$event_name"
    --output-json "$output_json"
    --simulate-delay-seconds "$simulate_delay_seconds"
    --ci-fast-gate "$ci_fast_gate"
  )
else
  lane_command=(
    bash "$DEEP_LANE"
    --event-name "$event_name"
    --output-json "$output_json"
    --simulate-delay-seconds "$simulate_delay_seconds"
    --ci-fast-gate "$ci_fast_gate"
  )
fi

if [[ -n "$max_seconds" ]]; then
  lane_command+=(--max-seconds "$max_seconds")
fi

if [[ -n "$fail_scenarios" ]]; then
  lane_command+=(--fail-scenarios "$fail_scenarios")
fi

lane_output="$("${lane_command[@]}")"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$output_json" \
    --max-artifact-age-seconds "$max_artifact_age_seconds" 2>&1
)"

if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected partition/reconnect policy checker to emit status=ok" >&2
  exit 1
fi

for required_ref in \
  "run_live_network_partition_reconnect_smoke_lane.sh" \
  "run_live_network_partition_reconnect_deep_lane.sh" \
  "select_live_network_partition_reconnect_lane.sh" \
  "check_live_network_partition_reconnect_policy.sh" \
  "run_live_network_partition_reconnect_contract_lane.sh"; do
  if ! grep -q "$required_ref" "$LIVE_NETWORK_DOC"; then
    echo "expected live-network wave doc to reference $required_ref" >&2
    exit 1
  fi
  if ! grep -q "$required_ref" "$GONOGO_DOC"; then
    echo "expected release go/no-go checklist to reference $required_ref" >&2
    exit 1
  fi
done

if ! printf '%s\n' "$lane_output" | grep -q "lane=$selected_lane"; then
  echo "expected partition/reconnect lane output to include selected lane marker" >&2
  exit 1
fi

echo "lane=$selected_lane"
echo "cadence=$cadence"
echo "policy=ok"
echo "live-network partition/reconnect contract lane tests passed."
