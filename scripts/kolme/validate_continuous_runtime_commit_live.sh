#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_continuous_runtime_commit_contract_lane.sh"

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
contract_report="$TMP_DIR/continuous-runtime-commit-contract-report.json"
run_output="$({
  bash "$RUNNER" --max-seconds 180 --output-json "$contract_report"
} 2>&1)"

if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected continuous runtime contract pass marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected continuous runtime contract GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^continuous_mode_status=verified$'; then
  echo "expected continuous mode status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^finality_recovery_status=verified$'; then
  echo "expected finality recovery status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^fail_closed_guard_status=verified$'; then
  echo "expected fail-closed guard status marker" >&2
  exit 1
fi

set +e
fail_closed_output="$({
  cd "$ROOT_DIR"
  cargo run -p kamn-node -- \
    --role processor \
    --runtime-mode kolme-live \
    --kolme-live-base-url http://127.0.0.1:3000 \
    --kolme-live-provider-hint kolme-fork-local \
    --kolme-live-signing-profile kolme-fork-secp256k1-v1 \
    --kolme-live-signer-key-source env-local \
    --daemon-max-ticks 2
} 2>&1)"
fail_closed_code=$?
set -e
if [ "$fail_closed_code" -eq 0 ]; then
  echo "expected missing paired cycle control drill to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_closed_output" | grep -q -- '--daemon-tick-interval-ms'; then
  echo "expected deterministic paired-control fail-closed marker" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "continuous runtime commit live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/continuous-runtime-commit-live-validation-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$report_json" <<JSON
{
  "schema_version": "kamn.kolme.continuous-runtime-commit.live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "continuous_runtime_contract_status": "verified",
  "evidence_bundle_status": "verified",
  "fail_closed_status": "verified",
  "fail_closed_reason_code": "paired_cycle_controls_required",
  "elapsed_seconds": ${elapsed_seconds}
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "continuous_runtime_contract_status=verified"
echo "evidence_bundle_status=verified"
echo "fail_closed_status=verified"
echo "fail_closed_reason_code=paired_cycle_controls_required"
