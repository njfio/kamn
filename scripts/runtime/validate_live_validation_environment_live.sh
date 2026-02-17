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
lane_report="$TMP_DIR/live-validation-environment-lane.json"
lane_output="$(
  bash "$ROOT_DIR/scripts/runtime/run_live_validation_environment_lane.sh" \
    --mode dry-run \
    --max-seconds 120 \
    --topology-max-seconds 60 \
    --kolme-max-seconds 120 \
    --output-json "$lane_report"
)"

if ! printf '%s\n' "$lane_output" | grep -q '^status=pass$'; then
  echo "expected live validation environment lane pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^final_decision=GO$'; then
  echo "expected live validation environment lane GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^topology_contract_status=verified$'; then
  echo "expected live validation environment topology marker" >&2
  exit 1
fi
if ! printf '%s\n' "$lane_output" | grep -q '^kolme_connectivity_contract_status=verified$'; then
  echo "expected live validation environment kolme marker" >&2
  exit 1
fi

set +e
fail_closed_output="$(
  bash "$ROOT_DIR/scripts/runtime/run_live_validation_environment_lane.sh" \
    --mode run \
    --max-seconds 120 \
    --topology-max-seconds 60 \
    --kolme-max-seconds 120 2>&1
)"
fail_closed_code=$?
set -e
if [ "$fail_closed_code" -eq 0 ]; then
  echo "expected live validation environment run mode without opt-in to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q 'run mode requires explicit local-only opt-in via KAMN_KOLME_LOCAL_HEAVY=1'; then
  echo "expected deterministic local-only opt-in failure marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "live validation environment live drill exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/live-validation-environment-live-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.runtime.live-validation-environment-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "lane_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "fail_closed_status": "verified",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "lane_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "fail_closed_status=verified"
