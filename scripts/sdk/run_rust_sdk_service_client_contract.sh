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
if [ "$max_seconds" -le 0 ]; then
  echo "max-seconds must be greater than zero" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
test_output_file="$TMP_DIR/service-client-contract-test.out"
set +e
(
  cd "$ROOT_DIR"
  cargo test -p kamn-sdk --test service_api_client
) >"$test_output_file" 2>&1
test_code=$?
set -e

if [ "$test_code" -ne 0 ]; then
  cat "$test_output_file" >&2
  echo "rust sdk service client contract target failed" >&2
  exit 1
fi

if ! grep -q 'test result: ok\.' "$test_output_file"; then
  cat "$test_output_file" >&2
  echo "expected service client contract test to report success" >&2
  exit 1
fi
if ! grep -q '4 passed; 0 failed' "$test_output_file"; then
  cat "$test_output_file" >&2
  echo "expected service client contract test pass count marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "rust sdk service client contract exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/rust-sdk-service-client-contract-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.sdk.rust-service-client-contract.v1",
  "status": "pass",
  "final_decision": "GO",
  "http_route_contract_status": "verified",
  "websocket_contract_status": "verified",
  "regression_guard_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "http_route_contract_status=verified"
echo "websocket_contract_status=verified"
echo "regression_guard_status=verified"
