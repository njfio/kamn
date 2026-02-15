#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-native-api-parity-live-proof-summary.json"
NONCE_COMMAND=""
BROADCAST_COMMAND=""
FINALITY_COMMAND=""
MAX_SECONDS=180
LOG_DIR="/tmp/kolme-local-native-api-parity-live-proof"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --mode)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --mode" >&2
        exit 1
      fi
      MODE="$2"
      shift 2
      ;;
    --output-json)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --output-json" >&2
        exit 1
      fi
      OUTPUT_JSON="$2"
      shift 2
      ;;
    --nonce-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --nonce-command" >&2
        exit 1
      fi
      NONCE_COMMAND="$2"
      shift 2
      ;;
    --broadcast-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --broadcast-command" >&2
        exit 1
      fi
      BROADCAST_COMMAND="$2"
      shift 2
      ;;
    --finality-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-command" >&2
        exit 1
      fi
      FINALITY_COMMAND="$2"
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
    --help|-h)
      cat <<'USAGE'
Usage: run_local_native_api_parity_live_proof_lane.sh [options]

Options:
  --mode dry-run|run              Print planned checks or execute local live proof checks.
  --output-json <path>            Deterministic summary report output path.
  --nonce-command <command>       Local nonce proof command for run mode.
  --broadcast-command <command>   Local broadcast proof command for run mode.
  --finality-command <command>    Local finality proof command for run mode.
  --max-seconds <n>               Max total runtime budget in seconds for run mode.
  --log-dir <path>                Directory for nonce/broadcast/finality command logs.
USAGE
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ "$MODE" != "dry-run" ] && [ "$MODE" != "run" ]; then
  echo "mode must be one of: dry-run, run" >&2
  exit 1
fi

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
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

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
local_only_enforced="true"

nonce_log="$LOG_DIR/nonce-proof.log"
broadcast_log="$LOG_DIR/broadcast-proof.log"
finality_log="$LOG_DIR/finality-proof.log"

planned_nonce_command="${NONCE_COMMAND:-<required-in-run-mode>}"
planned_broadcast_command="${BROADCAST_COMMAND:-<required-in-run-mode>}"
planned_finality_command="${FINALITY_COMMAND:-<required-in-run-mode>}"

record_check "nonce_probe" "$planned_nonce_command" "planned" "not_run"
record_check "broadcast_probe" "$planned_broadcast_command" "planned" "not_run"
record_check "finality_probe" "$planned_finality_command" "planned" "not_run"

run_proof_check() {
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

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"
  mkdir -p "$LOG_DIR"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "nonce_probe" "$planned_nonce_command" "fail" "local_opt_in_missing"
    record_check "broadcast_probe" "$planned_broadcast_command" "fail" "local_opt_in_missing"
    record_check "finality_probe" "$planned_finality_command" "fail" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  elif [ -z "$NONCE_COMMAND" ] || [ -z "$BROADCAST_COMMAND" ] || [ -z "$FINALITY_COMMAND" ]; then
    echo "run mode requires --nonce-command, --broadcast-command, and --finality-command" >&2
    record_check "nonce_probe" "$planned_nonce_command" "fail" "required_command_missing"
    record_check "broadcast_probe" "$planned_broadcast_command" "fail" "required_command_missing"
    record_check "finality_probe" "$planned_finality_command" "fail" "required_command_missing"
    overall_status="fail"
    reason_code="required_command_missing"
  else
    run_proof_check \
      "nonce_probe" \
      "$NONCE_COMMAND" \
      "$nonce_log" \
      "nonce_command_timeout" \
      "nonce_command_failed" \
      "$(( $(date +%s) - start_epoch ))"
    if [ "$overall_status" = "ok" ]; then
      run_proof_check \
        "broadcast_probe" \
        "$BROADCAST_COMMAND" \
        "$broadcast_log" \
        "broadcast_command_timeout" \
        "broadcast_command_failed" \
        "$(( $(date +%s) - start_epoch ))"
    fi
    if [ "$overall_status" = "ok" ]; then
      run_proof_check \
        "finality_probe" \
        "$FINALITY_COMMAND" \
        "$finality_log" \
        "finality_command_timeout" \
        "finality_command_failed" \
        "$(( $(date +%s) - start_epoch ))"
    fi
  fi

  elapsed_seconds="$(( $(date +%s) - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ] && [ "$reason_code" != "native_parity_budget_exceeded" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="native_parity_budget_exceeded"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    reason_code="native_parity_live_proof_passed"
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_native_api_parity_live_proof_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$local_only_enforced" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$planned_nonce_command" "$planned_broadcast_command" "$planned_finality_command" "$nonce_log" "$broadcast_log" "$finality_log" "$CHECK_FILE"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=$local_only_enforced"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
