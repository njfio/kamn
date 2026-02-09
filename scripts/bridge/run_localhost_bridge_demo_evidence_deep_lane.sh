#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_localhost_bridge_demo_evidence_contract_lane.sh"
RELAY_LANE_SCRIPT="$ROOT_DIR/scripts/bridge/run_localhost_bridge_relay_demo_contract_lane.sh"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
EVIDENCE_GENERATOR="$ROOT_DIR/scripts/bridge/generate_localhost_bridge_demo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/bridge/check_localhost_bridge_demo_policy.sh"

output_json=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
deep_relay_output="$TMP_DIR/localhost-bridge-relay-deep.out"
deep_replay_report="$TMP_DIR/localhost-bridge-replay-deep-report.json"
deep_bundle="$TMP_DIR/localhost-bridge-demo-evidence-deep-bundle.json"

bash "$CONTRACT_LANE" >/dev/null
bash "$RELAY_LANE_SCRIPT" >"$deep_relay_output"

bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" \
  --output-json "$deep_replay_report" >/dev/null

bundle_output="$(
  bash "$EVIDENCE_GENERATOR" \
    --output-file "$deep_bundle" \
    --lane deep \
    --relay-lane-output-file "$deep_relay_output" \
    --replay-report-file "$deep_replay_report" \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$bundle_output" | grep -q '^status=generated$'; then
  echo "expected localhost bridge demo deep evidence bundle generation to succeed" >&2
  exit 1
fi

if ! printf '%s\n' "$bundle_output" | grep -q '^final_decision=GO$'; then
  echo "expected localhost bridge demo deep evidence bundle final decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$deep_bundle")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected localhost bridge demo deep policy check to pass" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected localhost bridge demo deep policy final decision to be GO" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  cp "$deep_bundle" "$output_json"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 300 ]; then
  echo "localhost bridge demo evidence deep lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "localhost bridge demo evidence deep lane tests passed."
