#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RELAY_LANE_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_relay_demo_contract_lane.sh"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
EVIDENCE_GENERATOR="$ROOT_DIR/scripts/bridge/generate_localhost_bridge_demo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/bridge/check_localhost_bridge_demo_policy.sh"

skip_replay=false
replay_report_file=""
output_bundle=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-replay)
      skip_replay=true
      shift
      ;;
    --replay-report-file)
      replay_report_file="${2:-}"
      shift 2
      ;;
    --output-bundle)
      output_bundle="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ "$skip_replay" = true ] && [[ -z "$replay_report_file" ]]; then
  echo "--replay-report-file is required when --skip-replay is enabled" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
start_epoch="$(date +%s)"

relay_lane_output_file="$TMP_DIR/localhost-bridge-relay-contract.out"
bash "$RELAY_LANE_SCRIPT" >"$relay_lane_output_file"

if ! grep -q '^bridge_demo_signed_transport=pass$' "$relay_lane_output_file"; then
  echo "expected localhost bridge relay contract lane signed transport marker to pass" >&2
  exit 1
fi

if ! grep -q '^bridge_demo_relay_contracts=pass$' "$relay_lane_output_file"; then
  echo "expected localhost bridge relay contract lane relay contract marker to pass" >&2
  exit 1
fi

if [ "$skip_replay" = true ]; then
  if [ ! -f "$replay_report_file" ]; then
    echo "replay report file not found: $replay_report_file" >&2
    exit 1
  fi
else
  replay_report_file="$TMP_DIR/localhost-bridge-replay-contract-report.json"
  bash "$REPLAY_SCRIPT" \
    --fixture "$REPLAY_FIXTURE" \
    --suites "bridge_adapter" \
    --output-json "$replay_report_file" >/dev/null
fi

bundle_file="$TMP_DIR/localhost-bridge-demo-evidence-contract.json"
bundle_output="$(
  bash "$EVIDENCE_GENERATOR" \
    --output-file "$bundle_file" \
    --lane contract \
    --relay-lane-output-file "$relay_lane_output_file" \
    --replay-report-file "$replay_report_file" \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$bundle_output" | grep -q '^status=generated$'; then
  echo "expected localhost bridge demo evidence bundle generation to succeed" >&2
  exit 1
fi

if ! printf '%s\n' "$bundle_output" | grep -q '^final_decision=GO$'; then
  echo "expected localhost bridge demo evidence bundle final decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected localhost bridge demo evidence policy check to pass" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected localhost bridge demo evidence policy final decision to be GO" >&2
  exit 1
fi

if [[ -n "$output_bundle" ]]; then
  cp "$bundle_file" "$output_bundle"
fi

echo "localhost_bridge_demo_evidence=generated"
echo "localhost_bridge_demo_policy=ok"

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "localhost bridge demo evidence contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "localhost bridge demo evidence contract lane tests passed."
