#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

mode="contract"
languages="all"
fixture="$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json"
output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode)
      mode="${2:-}"
      shift 2
      ;;
    --languages)
      languages="${2:-}"
      shift 2
      ;;
    --fixture)
      fixture="${2:-}"
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
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [[ "$mode" != "contract" && "$mode" != "deep" ]]; then
  echo "mode must be one of: contract,deep" >&2
  exit 1
fi
if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi
if [[ "$mode" == "deep" && "$languages" != "all" ]]; then
  echo "deep mode only supports languages=all" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
register_report="$TMP_DIR/register-parity-report.json"
register_output="$({
  bash "$ROOT_DIR/scripts/sdk/run_sdk_parity_matrix.sh" \
    --fixture "$fixture" \
    --output-json "$register_report"
} 2>&1)"
if ! printf '%s\n' "$register_output" | grep -q '^status=pass;'; then
  echo "cross-language register parity matrix failed" >&2
  printf '%s\n' "$register_output" >&2
  exit 1
fi

if [[ "$mode" == "contract" ]]; then
  live_output="$({
    bash "$ROOT_DIR/scripts/sdk/run_live_transport_parity_contract_lane.sh" \
      --languages "$languages"
  } 2>&1)"
  if ! printf '%s\n' "$live_output" | grep -q 'live transport parity contract lane tests passed'; then
    echo "live transport parity contract lane did not emit success marker" >&2
    printf '%s\n' "$live_output" >&2
    exit 1
  fi
else
  live_output="$({
    bash "$ROOT_DIR/scripts/sdk/run_live_transport_parity_deep_lane.sh"
  } 2>&1)"
  if ! printf '%s\n' "$live_output" | grep -q 'live transport parity deep lane tests passed.'; then
    echo "live transport parity deep lane did not emit success marker" >&2
    printf '%s\n' "$live_output" >&2
    exit 1
  fi
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "cross-language sdk parity matrix exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/cross-language-sdk-parity-matrix-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.sdk.cross-language-parity.v1",
  "status": "pass",
  "final_decision": "GO",
  "mode": "${mode}",
  "languages": "${languages}",
  "fixture": "${fixture}",
  "register_parity_status": "verified",
  "live_transport_parity_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "mode=${mode}"
echo "register_parity_status=verified"
echo "live_transport_parity_status=verified"
