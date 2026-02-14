#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
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

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
config_file="$TMP_DIR/kamn-node-runtime.conf"
cat >"$config_file" <<'CONFIG'
role=listener
chain_id=kamn-config-file
chain_version=v0.2.0
storage_dir=./tmp/config-listener
sync_mode=archive
enable_gossip=false
output=json
CONFIG

config_only_output="$({
  cargo run -q -p kamn-node -- \
    --config-file "$config_file" \
    --output json;
} 2>&1)"
if ! printf '%s\n' "$config_only_output" | grep -q '"chain_id":"kamn-config-file"'; then
  echo "expected config-only probe to resolve chain_id from config file" >&2
  exit 1
fi
if ! printf '%s\n' "$config_only_output" | grep -q '"sync_mode":"archive"'; then
  echo "expected config-only probe to project sync_mode from config file" >&2
  exit 1
fi

env_override_output="$({
  KAMN_NODE_CHAIN_ID=kamn-env \
  cargo run -q -p kamn-node -- \
    --config-file "$config_file" \
    --output json;
} 2>&1)"
if ! printf '%s\n' "$env_override_output" | grep -q '"chain_id":"kamn-env"'; then
  echo "expected env override probe to project chain_id from KAMN_NODE_CHAIN_ID" >&2
  exit 1
fi

cli_override_output="$({
  KAMN_NODE_CHAIN_ID=kamn-env \
  cargo run -q -p kamn-node -- \
    --config-file "$config_file" \
    --chain-id kamn-cli \
    --output json;
} 2>&1)"
if ! printf '%s\n' "$cli_override_output" | grep -q '"chain_id":"kamn-cli"'; then
  echo "expected CLI override probe to win over env/config chain_id" >&2
  exit 1
fi
if ! printf '%s\n' "$cli_override_output" | grep -q '"role":"listener"'; then
  echo "expected CLI override probe to preserve config-driven role" >&2
  exit 1
fi

set +e
fail_closed_output="$({
  KAMN_NODE_SYNC_MODE=turbo \
  cargo run -q -p kamn-node -- \
    --config-file "$config_file";
} 2>&1)"
fail_closed_code=$?
set -e
if [ "$fail_closed_code" -eq 0 ]; then
  echo "expected invalid env override probe to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q 'invalid sync mode: turbo'; then
  echo "expected deterministic invalid sync-mode failure marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "config layering live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/config-layering-live-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.runtime.config-layering-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "layering_contract_status": "verified",
  "precedence_contract_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "invalid_sync_mode_override",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "layering_contract_status=verified"
echo "precedence_contract_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=invalid_sync_mode_override"
