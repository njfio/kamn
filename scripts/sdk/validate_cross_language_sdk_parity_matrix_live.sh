#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_cross_language_sdk_parity_matrix.sh"

output_json=""
max_seconds=240

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
live_report="$TMP_DIR/cross-language-sdk-parity-live-report.json"
run_output="$({
  bash "$RUNNER" \
    --mode contract \
    --languages python \
    --fixture "$ROOT_DIR/fixtures/sdk_parity/register_validation_cases.json" \
    --max-seconds 180 \
    --output-json "$live_report"
} 2>&1)"

if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected cross-language sdk parity runner pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected cross-language sdk parity runner GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^register_parity_status=verified$'; then
  echo "expected cross-language sdk parity runner register parity marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^live_transport_parity_status=verified$'; then
  echo "expected cross-language sdk parity runner live transport parity marker" >&2
  exit 1
fi

set +e
fail_closed_output="$({
  bash "$RUNNER" --mode invalid
} 2>&1)"
fail_closed_code=$?
set -e
if [ "$fail_closed_code" -eq 0 ]; then
  echo "expected invalid mode drill to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q 'mode must be one of: contract,deep'; then
  echo "expected deterministic invalid mode failure marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "cross-language sdk parity live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/cross-language-sdk-parity-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.sdk.cross-language-parity-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "matrix_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "invalid_mode",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "matrix_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=invalid_mode"
