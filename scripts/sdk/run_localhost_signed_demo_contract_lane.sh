#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF_USAGE'
Usage: run_localhost_signed_demo_contract_lane.sh [options]

Options:
  --output-json <path>   Write localhost signed demo contract lane report JSON.
  --max-seconds <n>      Runtime budget in seconds (default: 180 or env override).
  --help                 Show this help output.

Environment:
  KAMN_LOCALHOST_SIGNED_DEMO_CONTRACT_MAX_SECONDS
EOF_USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo.sh"
INTEGRATION_CONTRACT_LANE="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"

output_json=""
max_seconds="${KAMN_LOCALHOST_SIGNED_DEMO_CONTRACT_MAX_SECONDS:-180}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      if [ -z "$output_json" ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      if [ -z "$max_seconds" ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[1-9][0-9]*$ ]]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected localhost signed demo script to be executable" >&2
  exit 1
fi

if [ ! -x "$INTEGRATION_CONTRACT_LANE" ]; then
  echo "expected localhost signed integration contract lane script to be executable" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

demo_artifact="$TMP_DIR/localhost-signed-demo-artifact.json"
integration_report="$TMP_DIR/localhost-signed-integration-contract-report.json"

start_epoch="$(date +%s)"

demo_output="$(bash "$DEMO_SCRIPT" --output-json "$demo_artifact")"
if ! printf '%s\n' "$demo_output" | grep -q "localhost signed message demo completed."; then
  echo "expected localhost signed demo completion marker" >&2
  exit 1
fi
if ! printf '%s\n' "$demo_output" | grep -q "artifact_schema=kamn.sdk.localhost-signed.demo-receipt-artifact.v1"; then
  echo "expected localhost signed demo artifact schema marker" >&2
  exit 1
fi

integration_output="$(bash "$INTEGRATION_CONTRACT_LANE" --output-json "$integration_report")"
if ! printf '%s\n' "$integration_output" | grep -q "localhost signed integration contract lane tests passed."; then
  echo "expected localhost signed integration contract lane completion marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "localhost signed demo contract lane exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

if [ -n "$output_json" ]; then
  mkdir -p "$(dirname "$output_json")"
  python3 - \
    "$demo_artifact" \
    "$integration_report" \
    "$output_json" \
    "$elapsed_seconds" \
    "$max_seconds" <<'PY'
import json
import pathlib
import sys

demo_artifact_path = pathlib.Path(sys.argv[1])
integration_report_path = pathlib.Path(sys.argv[2])
output_path = pathlib.Path(sys.argv[3])
elapsed_seconds = int(sys.argv[4])
max_seconds = int(sys.argv[5])

demo_artifact = json.loads(demo_artifact_path.read_text(encoding="utf-8"))
integration_report = json.loads(integration_report_path.read_text(encoding="utf-8"))

if demo_artifact.get("schema_version") != "kamn.sdk.localhost-signed.demo-receipt-artifact.v1":
    raise SystemExit("unexpected localhost signed demo artifact schema")
if demo_artifact.get("status") != "pass":
    raise SystemExit("expected localhost signed demo artifact status=pass")

if integration_report.get("schema_version") != "kamn.sdk.localhost-signed.integration-contract.v1":
    raise SystemExit("unexpected localhost signed integration report schema")
if integration_report.get("status") != "pass":
    raise SystemExit("expected localhost signed integration report status=pass")

payload = {
    "schema_version": "kamn.sdk.localhost-signed.demo-contract.v1",
    "status": "pass",
    "suite": "localhost_signed_demo_contract_lane",
    "demo_artifact_schema": "kamn.sdk.localhost-signed.demo-receipt-artifact.v1",
    "integration_report_schema": "kamn.sdk.localhost-signed.integration-contract.v1",
    "demo_status": "pass",
    "integration_status": "pass",
    "demo_success_marker": "localhost signed message demo completed.",
    "integration_success_marker": "localhost signed integration contract lane tests passed.",
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": "within_budget",
    "reason_codes": ["none"],
}
output_path.write_text(json.dumps(payload, separators=(",", ":")), encoding="utf-8")
PY
  echo "localhost_signed_demo_contract_report=$output_json"
fi

echo "localhost_signed_demo_status=pass"
echo "localhost_signed_integration_status=pass"
echo "localhost signed demo contract lane tests passed."
