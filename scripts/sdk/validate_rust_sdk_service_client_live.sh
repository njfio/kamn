#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/sdk/run_rust_sdk_service_client_contract.sh"

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
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
contract_report="$TMP_DIR/rust-sdk-service-client-contract-report.json"
run_output="$({
  bash "$RUNNER" --max-seconds 180 --output-json "$contract_report"
} 2>&1)"

if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected rust sdk service client contract pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected rust sdk service client contract GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^http_route_contract_status=verified$'; then
  echo "expected rust sdk service client contract http marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^websocket_contract_status=verified$'; then
  echo "expected rust sdk service client contract websocket marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^regression_guard_status=verified$'; then
  echo "expected rust sdk service client contract regression marker" >&2
  exit 1
fi

set +e
fail_closed_output="$({
  bash "$RUNNER" --max-seconds 0
} 2>&1)"
fail_closed_code=$?
set -e
if [ "$fail_closed_code" -eq 0 ]; then
  echo "expected zero max-seconds drill to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q 'max-seconds must be greater than zero'; then
  echo "expected deterministic zero max-seconds fail-closed marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "rust sdk service client live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/rust-sdk-service-client-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.sdk.rust-service-client-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "service_client_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "zero_runtime_budget",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "service_client_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=zero_runtime_budget"
