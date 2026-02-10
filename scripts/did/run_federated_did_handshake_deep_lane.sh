#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/did/run_federated_did_handshake_contract_lane.sh"
MATRIX_SCRIPT="$ROOT_DIR/scripts/did/run_federated_did_handshake_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/federated_did_handshake/partition_replay_cases.json"
POLICY_CHECKER="$ROOT_DIR/scripts/did/check_federated_did_handshake_deep_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

usage() {
  cat <<'USAGE'
Usage:
  KAMN_FEDERATED_DID_HANDSHAKE_DEEP_CADENCE=scheduled \
    bash scripts/did/run_federated_did_handshake_deep_lane.sh \
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

event_name="${GITHUB_EVENT_NAME:-schedule}"
output_json="$ROOT_DIR/federated-did-handshake-report.json"
max_seconds="${KAMN_FEDERATED_DID_HANDSHAKE_DEEP_MAX_SECONDS:-180}"
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
  fail "expected federated DID handshake contract lane runner to be executable"
fi
if [ ! -x "$MATRIX_SCRIPT" ]; then
  fail "expected federated DID handshake matrix runner to be executable"
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  fail "expected federated DID handshake deep policy checker to be executable"
fi
if [ ! -f "$FIXTURE_FILE" ]; then
  fail "expected federated DID handshake fixture file to exist"
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

configured_cadence="${KAMN_FEDERATED_DID_HANDSHAKE_DEEP_CADENCE:-}"
if [[ -n "$configured_cadence" ]]; then
  if [[ "$configured_cadence" != "scheduled" && "$configured_cadence" != "manual" ]]; then
    fail "KAMN_FEDERATED_DID_HANDSHAKE_DEEP_CADENCE must be scheduled or manual"
  fi
  if [[ "$configured_cadence" != "$cadence" ]]; then
    fail "configured deep cadence does not match event-derived cadence"
  fi
fi

mkdir -p "$(dirname "$output_json")"
matrix_report="$TMP_DIR/federated-did-handshake-matrix-report.json"
start_epoch="$(date +%s)"
status="pass"
contract_lane_status="pass"
matrix_status="pass"
reason_codes=()

if [ "$skip_contract_tests" = true ]; then
  contract_output="contract lane skipped via --skip-contract-tests"
  contract_code=0
else
  set +e
  contract_output="$(bash "$CONTRACT_LANE" 2>&1)"
  contract_code=$?
  set -e
fi

if [ "$contract_code" -ne 0 ]; then
  contract_lane_status="fail"
  status="fail"
  reason_codes+=("contract_lane_failed")
fi

set +e
matrix_output="$(
  python3 "$MATRIX_SCRIPT" \
    --fixture "$FIXTURE_FILE" \
    --output-json "$matrix_report" 2>&1
)"
matrix_code=$?
set -e

if [ "$matrix_code" -ne 0 ] || ! printf '%s\n' "$matrix_output" | grep -q '^status=pass;'; then
  matrix_status="fail"
  status="fail"
  reason_codes+=("matrix_failed")
fi

matrix_counts="$(
  python3 - "$matrix_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.is_file():
    print("0 0")
    raise SystemExit(0)

payload = json.loads(path.read_text(encoding="utf-8"))
print(f"{int(payload.get('case_count', 0))} {int(payload.get('failed_count', 0))}")
PY
)"
matrix_case_count="$(printf '%s\n' "$matrix_counts" | awk '{print $1}')"
matrix_failed_count="$(printf '%s\n' "$matrix_counts" | awk '{print $2}')"

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
budget_status="within"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  budget_status="exceeded"
  status="fail"
  reason_codes+=("runtime_budget_exceeded")
fi

reason_codes_json="$(
  python3 - "${reason_codes[@]:-}" <<'PY'
import json
import sys

values = sorted({value for value in sys.argv[1:] if value})
print(json.dumps(values))
PY
)"

final_decision="GO"
if [ "$status" != "pass" ]; then
  final_decision="NO-GO"
fi

python3 - "$output_json" "$event_name" "$cadence" "$contract_lane_status" "$matrix_status" "$matrix_case_count" "$matrix_failed_count" "$matrix_report" "$elapsed_seconds" "$max_seconds" "$budget_status" "$reason_codes_json" "$final_decision" "$skip_contract_tests" <<'PY'
import json
import pathlib
import sys

(
    output_json,
    event_name,
    cadence,
    contract_lane_status,
    matrix_status,
    matrix_case_count,
    matrix_failed_count,
    matrix_report_file,
    elapsed_seconds,
    max_seconds,
    budget_status,
    reason_codes_json,
    final_decision,
    skip_contract_tests,
) = sys.argv[1:]

payload = {
    "schema_version": "kamn.did.federated-handshake.deep-summary.v1",
    "event_name": event_name,
    "cadence": cadence,
    "contract_lane_status": contract_lane_status,
    "matrix_status": matrix_status,
    "matrix_case_count": int(matrix_case_count),
    "matrix_failed_count": int(matrix_failed_count),
    "matrix_report_file": matrix_report_file,
    "elapsed_seconds": int(elapsed_seconds),
    "max_seconds": int(max_seconds),
    "budget_status": budget_status,
    "reason_codes": json.loads(reason_codes_json),
    "final_decision": final_decision,
    "policy_status": "pending",
    "skip_contract_tests": skip_contract_tests == "true",
}

pathlib.Path(output_json).write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
policy_output="$(bash "$POLICY_CHECKER" --report-file "$output_json" 2>&1)"
policy_code=$?
set -e

policy_status="pass"
if [ "$policy_code" -ne 0 ]; then
  policy_status="fail"
  status="fail"
  reason_codes+=("policy_check_failed")
  final_decision="NO-GO"
fi

reason_codes_json="$(
  python3 - "${reason_codes[@]:-}" <<'PY'
import json
import sys

values = sorted({value for value in sys.argv[1:] if value})
print(json.dumps(values))
PY
)"

python3 - "$output_json" "$policy_status" "$final_decision" "$reason_codes_json" <<'PY'
import json
import pathlib
import sys

output_json, policy_status, final_decision, reason_codes_json = sys.argv[1:]
path = pathlib.Path(output_json)
payload = json.loads(path.read_text(encoding="utf-8"))
payload["policy_status"] = policy_status
payload["final_decision"] = final_decision
payload["reason_codes"] = json.loads(reason_codes_json)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

if [ "$status" != "pass" ]; then
  printf '%s\n' "$contract_output" >&2
  printf '%s\n' "$matrix_output" >&2
  printf '%s\n' "$policy_output" >&2
  fail "federated DID handshake deep lane failed closed: $(IFS=,; echo "${reason_codes[*]}")"
fi

echo "federated DID handshake deep lane tests passed."
