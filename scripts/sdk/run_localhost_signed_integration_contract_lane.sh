#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
HARNESS_RUNNER="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_harness.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/sdk/check_localhost_signed_integration_evidence_policy.sh"
LIVE_NETWORK_DOC="$ROOT_DIR/docs/planning/live-network-wave.md"
README_FILE="$ROOT_DIR/README.md"

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

if [ ! -x "$HARNESS_RUNNER" ]; then
  echo "expected localhost signed integration harness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected localhost signed integration evidence policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$LIVE_NETWORK_DOC" ]; then
  echo "expected live-network planning doc to exist" >&2
  exit 1
fi

if [ ! -f "$README_FILE" ]; then
  echo "expected README.md to exist" >&2
  exit 1
fi

max_seconds="${KAMN_LOCALHOST_SIGNED_INTEGRATION_CONTRACT_MAX_SECONDS:-120}"
if [[ ! "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "contract max seconds must be a positive integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
start_epoch="$(date +%s)"

success_report="$TMP_DIR/success.json"
success_output="$(
  bash "$HARNESS_RUNNER" \
    --scenario success \
    --output-json "$success_report"
)"
if ! printf '%s\n' "$success_output" | grep -Fq "status=pass; scenario=success; reason_code=none;"; then
  echo "expected localhost signed integration success scenario to pass" >&2
  exit 1
fi

signature_report="$TMP_DIR/signature-mismatch.json"
signature_output="$(
  bash "$HARNESS_RUNNER" \
    --scenario signature-mismatch \
    --output-json "$signature_report"
)"
if ! printf '%s\n' "$signature_output" | grep -Fq "status=pass; scenario=signature-mismatch; reason_code=signature_mismatch_detected;"; then
  echo "expected localhost signed integration signature mismatch scenario to pass" >&2
  exit 1
fi

timeout_report="$TMP_DIR/timeout.json"
timeout_output="$(
  bash "$HARNESS_RUNNER" \
    --scenario timeout \
    --timeout-seconds 1 \
    --output-json "$timeout_report"
)"
if ! printf '%s\n' "$timeout_output" | grep -Fq "status=pass; scenario=timeout; reason_code=listener_timeout_detected;"; then
  echo "expected localhost signed integration timeout scenario to pass" >&2
  exit 1
fi

summary_report="$TMP_DIR/localhost-signed-integration-contract.json"
python3 - "$success_report" "$signature_report" "$timeout_report" "$summary_report" <<'PY'
import json
import pathlib
import sys

success = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
signature = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
timeout = json.loads(pathlib.Path(sys.argv[3]).read_text(encoding="utf-8"))
output_file = pathlib.Path(sys.argv[4])

summary = {
    "schema_version": "kamn.sdk.localhost-signed.integration-contract.v1",
    "status": "pass",
    "success_scenario_status": success["status"],
    "signature_mismatch_scenario_status": signature["status"],
    "timeout_scenario_status": timeout["status"],
    "signature_mismatch_reason_code": signature["reason_code"],
    "timeout_reason_code": timeout["reason_code"],
    "success_elapsed_seconds": success["elapsed_seconds"],
    "signature_mismatch_elapsed_seconds": signature["elapsed_seconds"],
    "timeout_elapsed_seconds": timeout["elapsed_seconds"],
}
output_file.write_text(json.dumps(summary, separators=(",", ":")), encoding="utf-8")
PY

policy_output="$(bash "$POLICY_CHECKER" --report-file "$summary_report")"
if ! printf '%s\n' "$policy_output" | grep -Fq "status=ok"; then
  echo "expected localhost signed integration evidence policy check to pass" >&2
  exit 1
fi

if ! grep -Fq "run_localhost_signed_integration_contract_lane.sh" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network planning doc to reference localhost signed integration contract lane" >&2
  exit 1
fi

if ! grep -Fq "check_localhost_signed_integration_evidence_policy.sh" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network planning doc to reference localhost signed integration evidence policy checker" >&2
  exit 1
fi

if ! grep -Fq "/tmp/localhost-signed-integration-contract-report.json" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network planning doc to reference localhost signed integration report artifact path" >&2
  exit 1
fi

if ! grep -Fq "run_localhost_signed_integration_contract_lane.sh" "$README_FILE"; then
  echo "expected README to reference localhost signed integration contract lane command" >&2
  exit 1
fi

if ! grep -Fq "check_localhost_signed_integration_evidence_policy.sh" "$README_FILE"; then
  echo "expected README to reference localhost signed integration evidence policy checker command" >&2
  exit 1
fi

if ! grep -Fq "/tmp/localhost-signed-integration-contract-report.json" "$README_FILE"; then
  echo "expected README to reference localhost signed integration report artifact path" >&2
  exit 1
fi

if [[ -n "$output_json" ]]; then
  cp "$summary_report" "$output_json"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "localhost signed integration contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

echo "localhost_signed_integration_success=pass"
echo "localhost_signed_integration_signature_mismatch=pass"
echo "localhost_signed_integration_timeout=pass"
echo "localhost_signed_integration_policy=ok"
echo "localhost signed integration contract lane tests passed."
