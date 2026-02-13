#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPLAY_FIXTURE="$ROOT_DIR/fixtures/bridge_replay/replay_validation_cases.json"
REPLAY_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_replay_matrix.sh"
REDACTION_CHECK_SCRIPT="$ROOT_DIR/scripts/bridge/run_bridge_credential_redaction_check.py"
EVIDENCE_GENERATOR="$ROOT_DIR/scripts/bridge/generate_bridge_replay_redaction_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/bridge/check_bridge_replay_redaction_policy.sh"

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

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"

if [ "$skip_replay" = true ] && [[ -z "$replay_report_file" ]]; then
  echo "--replay-report-file is required when --skip-replay is enabled" >&2
  exit 1
fi

if [ "$skip_replay" = true ]; then
  if [ ! -f "$replay_report_file" ]; then
    echo "replay report file not found: $replay_report_file" >&2
    exit 1
  fi
else
  replay_report_file="$TMP_DIR/bridge-replay-contract-report.json"
  bash "$REPLAY_SCRIPT" \
    --fixture "$REPLAY_FIXTURE" \
    --suites "bridge_adapter,telegram_bridge,discord_bridge" \
    --output-json "$replay_report_file" >/dev/null
fi

redaction_report_file="$TMP_DIR/bridge-redaction-contract-report.json"
redaction_stdout_file="$TMP_DIR/bridge-redaction-contract.out"
bundle_file="$TMP_DIR/bridge-replay-redaction-contract-bundle.json"

TELEGRAM_TOKEN="telegram_contract_token_v6c8m1p2q4r9"
DISCORD_TOKEN="discord_contract_token_h7j2k5l8z1x3"
CROSS_CHAIN_TOKEN="crosschain_contract_token_n4b7v2c9m5q1"

python3 "$REDACTION_CHECK_SCRIPT" \
  --mode contract \
  --output-json "$redaction_report_file" \
  --telegram-token "$TELEGRAM_TOKEN" \
  --discord-token "$DISCORD_TOKEN" \
  --cross-chain-token "$CROSS_CHAIN_TOKEN" >"$redaction_stdout_file"

if ! grep -q '^status=pass$' "$redaction_stdout_file"; then
  echo "expected bridge redaction checker contract mode to pass" >&2
  exit 1
fi

for secret in "$TELEGRAM_TOKEN" "$DISCORD_TOKEN" "$CROSS_CHAIN_TOKEN"; do
  if grep -q "$secret" "$redaction_report_file"; then
    echo "redaction contract report leaked raw credential material" >&2
    exit 1
  fi
  if grep -q "$secret" "$redaction_stdout_file"; then
    echo "redaction contract output leaked raw credential material" >&2
    exit 1
  fi
done

bundle_output="$(
  bash "$EVIDENCE_GENERATOR" \
    --output-file "$bundle_file" \
    --lane contract \
    --replay-report-file "$replay_report_file" \
    --redaction-report-file "$redaction_report_file" \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$bundle_output" | grep -q '^status=generated$'; then
  echo "expected bridge replay/redaction evidence bundle generation to succeed" >&2
  exit 1
fi

if ! printf '%s\n' "$bundle_output" | grep -q '^final_decision=GO$'; then
  echo "expected bridge replay/redaction contract bundle final decision to be GO" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected bridge replay/redaction policy check to pass" >&2
  exit 1
fi

if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected bridge replay/redaction policy checker final decision to be GO" >&2
  exit 1
fi

if [[ -n "$output_bundle" ]]; then
  cp "$bundle_file" "$output_bundle"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt 120 ]; then
  echo "bridge replay/redaction contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "bridge replay redaction contract lane tests passed."
