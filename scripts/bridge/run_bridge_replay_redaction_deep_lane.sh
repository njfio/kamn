#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/bridge/run_bridge_replay_redaction_contract_lane.sh"
REDACTION_CHECK_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credential_redaction_check.py"
EVIDENCE_GENERATOR="$ROOT_DIR/scripts/bridge/generate_bridge_replay_redaction_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/bridge/check_bridge_replay_redaction_policy.sh"

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

deep_replay_report="$TMP_DIR/bridge-replay-deep-report.json"
deep_redaction_report="$TMP_DIR/bridge-redaction-deep-report.json"
deep_redaction_stdout="$TMP_DIR/bridge-redaction-deep.out"
deep_bundle="$TMP_DIR/bridge-replay-redaction-deep-bundle.json"

bash "$CONTRACT_LANE" >/dev/null

bash "$REPLAY_SCRIPT" \
  --fixture "$REPLAY_FIXTURE" \
  --suites "bridge_adapter,telegram_bridge,discord_bridge,cross_chain_bridge" \
  --output-json "$deep_replay_report" >/dev/null

TELEGRAM_TOKEN="telegram_deep_token_m8q3v6x1h4n7"
DISCORD_TOKEN="discord_deep_token_p2c5k8r1t6y4"
CROSS_CHAIN_TOKEN="crosschain_deep_token_z9b4m7n2q5v8"

python3 "$REDACTION_CHECK_SCRIPT" \
  --mode deep \
  --output-json "$deep_redaction_report" \
  --telegram-token "$TELEGRAM_TOKEN" \
  --discord-token "$DISCORD_TOKEN" \
  --cross-chain-token "$CROSS_CHAIN_TOKEN" >"$deep_redaction_stdout"

if ! grep -q '^status=pass$' "$deep_redaction_stdout"; then
  echo "expected bridge redaction checker deep mode to pass" >&2
  exit 1
fi

if ! grep -q '"sample_count": 128' "$deep_redaction_report"; then
  echo "expected deep redaction report to include sample-count contract" >&2
  exit 1
fi

bundle_output="$(
  bash "$EVIDENCE_GENERATOR" \
    --output-file "$deep_bundle" \
    --lane deep \
    --replay-report-file "$deep_replay_report" \
    --redaction-report-file "$deep_redaction_report" \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$bundle_output" | grep -q '^status=generated$'; then
  echo "expected deep replay/redaction evidence bundle generation to succeed" >&2
  exit 1
fi

if ! printf '%s\n' "$bundle_output" | grep -q '^final_decision=GO$'; then
  echo "expected deep replay/redaction bundle final decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$deep_bundle")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected deep replay/redaction policy check to pass" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  cp "$deep_bundle" "$output_json"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 300 ]; then
  echo "bridge replay/redaction deep lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "bridge replay redaction deep lane tests passed."
