#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_JSON="/tmp/kolme-fast-gate-native-api-parity-summary.json"
MAX_SECONDS="${KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS:-120}"
LOG_DIR="/tmp/kolme-fast-gate-native-api-parity"
CI_FAST_GATE="PASS"
NONCE_BROADCAST_COMMAND="bash \"$ROOT_DIR/scripts/kolme/run_nonce_broadcast_parity_contract_lane.sh\""
NOTIFICATIONS_COMMAND="bash \"$ROOT_DIR/scripts/kolme/run_notifications_consumer_contract_lane.sh\""
BLOCK_FALLBACK_COMMAND="bash \"$ROOT_DIR/scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh\""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --log-dir)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --log-dir" >&2
        exit 1
      fi
      LOG_DIR="$2"
      shift 2
      ;;
    --ci-fast-gate)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --ci-fast-gate" >&2
        exit 1
      fi
      CI_FAST_GATE="$2"
      shift 2
      ;;
    --nonce-broadcast-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --nonce-broadcast-command" >&2
        exit 1
      fi
      NONCE_BROADCAST_COMMAND="$2"
      shift 2
      ;;
    --notifications-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --notifications-command" >&2
        exit 1
      fi
      NOTIFICATIONS_COMMAND="$2"
      shift 2
      ;;
    --block-fallback-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --block-fallback-command" >&2
        exit 1
      fi
      BLOCK_FALLBACK_COMMAND="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_fast_gate_native_api_parity_contract_lane.sh [options]

Options:
  --output-json <path>              Deterministic summary report output path.
  --max-seconds <n>                 Max runtime budget in seconds.
  --log-dir <path>                  Directory for lane command logs.
  --ci-fast-gate PASS|FAIL          Fast-gate policy marker.
  --nonce-broadcast-command <cmd>   Override nonce/broadcast parity contract command.
  --notifications-command <cmd>     Override notifications consumer contract command.
  --block-fallback-command <cmd>    Override block fallback contract command.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if [ "$CI_FAST_GATE" != "PASS" ] && [ "$CI_FAST_GATE" != "FAIL" ]; then
  echo "ci-fast-gate must be one of: PASS, FAIL" >&2
  exit 1
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  local reason="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason" >>"$CHECK_FILE"
}

run_bounded_check() {
  local check_id="$1"
  local command="$2"
  local log_path="$3"
  local timeout_reason="$4"
  local failed_reason="$5"
  local elapsed_now="$6"

  local remaining="$(( MAX_SECONDS - elapsed_now ))"
  if [ "$remaining" -le 0 ]; then
    record_check "$check_id" "$command" "fail" "native_parity_budget_exceeded"
    overall_status="fail"
    reason_code="native_parity_budget_exceeded"
    return
  fi

  local exit_code=0
  set +e
  timeout "$remaining" bash -lc "$command" >"$log_path" 2>&1
  exit_code=$?
  set -e

  if [ "$exit_code" -eq 0 ]; then
    record_check "$check_id" "$command" "pass" "passed"
    return
  fi
  if [ "$exit_code" -eq 124 ]; then
    record_check "$check_id" "$command" "fail" "$timeout_reason"
    overall_status="fail"
    reason_code="$timeout_reason"
    return
  fi
  record_check "$check_id" "$command" "fail" "$failed_reason"
  overall_status="fail"
  reason_code="$failed_reason"
}

overall_status="ok"
reason_code="fast_gate_native_api_parity_passed"
budget_status="within_budget"
elapsed_seconds=0

nonce_log="$LOG_DIR/nonce-broadcast-contract.log"
notifications_log="$LOG_DIR/notifications-consumer-contract.log"
block_fallback_log="$LOG_DIR/block-fallback-contract.log"

start_epoch="$(date +%s)"
mkdir -p "$LOG_DIR"

if [ "$CI_FAST_GATE" = "FAIL" ]; then
  record_check "nonce_broadcast_contract" "$NONCE_BROADCAST_COMMAND" "fail" "ci_fast_gate_failed"
  record_check "notifications_consumer_contract" "$NOTIFICATIONS_COMMAND" "fail" "ci_fast_gate_failed"
  record_check "block_fallback_contract" "$BLOCK_FALLBACK_COMMAND" "fail" "ci_fast_gate_failed"
  overall_status="fail"
  reason_code="ci_fast_gate_failed"
  budget_status="not_run"
else
  run_bounded_check \
    "nonce_broadcast_contract" \
    "$NONCE_BROADCAST_COMMAND" \
    "$nonce_log" \
    "nonce_broadcast_contract_timeout" \
    "nonce_broadcast_contract_failed" \
    "$(( $(date +%s) - start_epoch ))"

  if [ "$overall_status" = "ok" ]; then
    run_bounded_check \
      "notifications_consumer_contract" \
      "$NOTIFICATIONS_COMMAND" \
      "$notifications_log" \
      "notifications_consumer_contract_timeout" \
      "notifications_consumer_contract_failed" \
      "$(( $(date +%s) - start_epoch ))"
  else
    record_check "notifications_consumer_contract" "$NOTIFICATIONS_COMMAND" "fail" "skipped_due_prior_failure"
  fi

  if [ "$overall_status" = "ok" ]; then
    run_bounded_check \
      "block_fallback_contract" \
      "$BLOCK_FALLBACK_COMMAND" \
      "$block_fallback_log" \
      "block_fallback_contract_timeout" \
      "block_fallback_contract_failed" \
      "$(( $(date +%s) - start_epoch ))"
  else
    record_check "block_fallback_contract" "$BLOCK_FALLBACK_COMMAND" "fail" "skipped_due_prior_failure"
  fi
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  budget_status="exceeded_budget"
  if [ "$overall_status" = "ok" ]; then
    overall_status="fail"
    reason_code="native_parity_budget_exceeded"
  fi
fi

if [ "$overall_status" = "ok" ]; then
  reason_code="fast_gate_native_api_parity_passed"
fi

python3 - "$OUTPUT_JSON" "$overall_status" "$reason_code" "$CI_FAST_GATE" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$NONCE_BROADCAST_COMMAND" "$NOTIFICATIONS_COMMAND" "$BLOCK_FALLBACK_COMMAND" "$nonce_log" "$notifications_log" "$block_fallback_log" "$CHECK_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
status = sys.argv[2]
reason_code = sys.argv[3]
ci_fast_gate = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
budget_status = sys.argv[7]
nonce_broadcast_command = sys.argv[8]
notifications_command = sys.argv[9]
block_fallback_command = sys.argv[10]
nonce_log = sys.argv[11]
notifications_log = sys.argv[12]
block_fallback_log = sys.argv[13]
checks_path = pathlib.Path(sys.argv[14])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason,
        }
    )

summary = {
    "schema_version": "kamn.kolme.fast-gate-native-api-parity-summary.v1",
    "status": status,
    "reason_code": reason_code,
    "ci_fast_gate": ci_fast_gate,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "nonce_broadcast_command": nonce_broadcast_command,
    "notifications_command": notifications_command,
    "block_fallback_command": block_fallback_command,
    "checks": checks,
    "artifact_paths": [
        nonce_log,
        notifications_log,
        block_fallback_log,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
