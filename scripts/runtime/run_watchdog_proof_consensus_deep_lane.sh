#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/runtime/run_watchdog_proof_consensus_contract_lane.sh"
GENERATOR="$ROOT_DIR/scripts/runtime/generate_watchdog_proof_consensus_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_watchdog_proof_consensus_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE=scheduled \
    bash scripts/runtime/run_watchdog_proof_consensus_deep_lane.sh \
      [--event-name schedule|workflow_dispatch] \
      [--output-json <path>] \
      [--max-seconds <int>] \
      [--skip-contract-tests]
USAGE
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

run_anomaly_case() {
  local consensus_status="$1"
  local message_id="$2"
  local artifact_id="$3"
  local valid_count="$4"
  local invalid_count="$5"
  local replay_count="$6"
  local cadence="$7"
  local max_seconds="$8"
  local bundle_file="$TMP_DIR/watchdog-proof-consensus-${consensus_status}.json"

  local generator_output
  generator_output="$(
    bash "$GENERATOR" \
      --output-file "$bundle_file" \
      --message-id "$message_id" \
      --artifact-id "$artifact_id" \
      --consensus-status "$consensus_status" \
      --required-quorum 2 \
      --valid-attestation-count "$valid_count" \
      --invalid-attestation-count "$invalid_count" \
      --replay-attestation-count "$replay_count" \
      --cadence "$cadence" \
      --runtime-seconds 4 \
      --max-seconds "$max_seconds" \
      --evidence-complete true \
      --ci-fast-gate PASS
  )"
  if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=NO-GO$"; then
    fail "expected watchdog proof consensus ${consensus_status} anomaly evidence to produce NO-GO"
  fi

  local policy_output
  policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file")"
  if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=NO-GO$"; then
    fail "expected watchdog proof consensus ${consensus_status} anomaly policy check to produce NO-GO"
  fi
}

event_name="${GITHUB_EVENT_NAME:-schedule}"
output_json="$ROOT_DIR/watchdog-proof-consensus-deep-summary.json"
max_seconds="${KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_MAX_SECONDS:-180}"
skip_contract_tests=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --event-name)
      event_name="${2:-}"
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
    --skip-contract-tests)
      skip_contract_tests=true
      shift
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  fail "--max-seconds must be a non-negative integer"
fi
if [ "$max_seconds" -eq 0 ]; then
  fail "--max-seconds must be greater than zero"
fi

if [ ! -x "$CONTRACT_LANE" ]; then
  fail "watchdog proof consensus contract lane is not executable"
fi
if [ ! -x "$GENERATOR" ]; then
  fail "watchdog proof consensus evidence generator is not executable"
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  fail "watchdog proof consensus policy checker is not executable"
fi

cadence=""
case "$event_name" in
  schedule)
    cadence="scheduled"
    ;;
  workflow_dispatch)
    cadence="manual"
    ;;
  *)
    fail "scheduled/manual-only cadence policy requires event schedule or workflow_dispatch"
    ;;
esac

configured_cadence="${KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE:-}"
if [[ -n "$configured_cadence" ]]; then
  if [[ "$configured_cadence" != "scheduled" && "$configured_cadence" != "manual" ]]; then
    fail "KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE must be scheduled or manual"
  fi
  if [[ "$configured_cadence" != "$cadence" ]]; then
    fail "configured deep cadence does not match event-derived cadence"
  fi
fi

start_epoch="$(date +%s)"
status="pass"
failure_reason=""

contract_bundle="$TMP_DIR/watchdog-proof-consensus-contract.json"
contract_lane_status="pass"
set +e
if [ "$skip_contract_tests" = true ]; then
  contract_output="$(bash "$CONTRACT_LANE" --skip-tests --output-file "$contract_bundle" 2>&1)"
else
  contract_output="$(bash "$CONTRACT_LANE" --output-file "$contract_bundle" 2>&1)"
fi
contract_code=$?
set -e
if [ "$contract_code" -ne 0 ]; then
  contract_lane_status="fail"
  status="fail"
  failure_reason="watchdog proof consensus contract lane failed"
fi

if [ "$status" = "pass" ]; then
  run_anomaly_case ConsensusInvalid \
    "urn:uuid:watchdog-proof-invalid-996" \
    "artifact-watchdog-invalid-996" \
    1 \
    1 \
    0 \
    "$cadence" \
    "$max_seconds"
  run_anomaly_case ConsensusReplay \
    "urn:uuid:watchdog-proof-replay-996" \
    "artifact-watchdog-replay-996" \
    1 \
    0 \
    1 \
    "$cadence" \
    "$max_seconds"
  run_anomaly_case ValidatorMismatch \
    "urn:uuid:watchdog-proof-mismatch-996" \
    "artifact-watchdog-mismatch-996" \
    1 \
    1 \
    0 \
    "$cadence" \
    "$max_seconds"
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
budget_status="within"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  budget_status="exceeded"
  status="fail"
  failure_reason="watchdog proof consensus deep lane exceeded runtime budget"
fi

final_decision="GO"
if [ "$status" != "pass" ]; then
  final_decision="NO-GO"
fi

mkdir -p "$(dirname "$output_json")"
python3 - "$output_json" "$event_name" "$cadence" "$contract_lane_status" "$elapsed_seconds" "$max_seconds" "$budget_status" "$skip_contract_tests" "$final_decision" "$failure_reason" <<'PY'
import json
import pathlib
import sys

(
    output_json,
    event_name,
    cadence,
    contract_lane_status,
    elapsed_seconds,
    max_seconds,
    budget_status,
    skip_contract_tests,
    final_decision,
    failure_reason,
) = sys.argv[1:]

payload = {
    "schema_version": "kamn.runtime.watchdog-proof-consensus-deep-summary.v1",
    "event_name": event_name,
    "cadence": cadence,
    "contract_lane_status": contract_lane_status,
    "anomaly_matrix_cases": [
        "ConsensusInvalid",
        "ConsensusReplay",
        "ValidatorMismatch",
    ],
    "elapsed_seconds": int(elapsed_seconds),
    "max_seconds": int(max_seconds),
    "budget_status": budget_status,
    "skip_contract_tests": skip_contract_tests == "true",
    "final_decision": final_decision,
}
if failure_reason:
    payload["failure_reason"] = failure_reason

pathlib.Path(output_json).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [ "$status" != "pass" ]; then
  printf '%s\n' "$failure_reason" >&2
  printf '%s\n' "$contract_output" >&2
  exit 1
fi

echo "watchdog proof consensus deep lane tests passed."
