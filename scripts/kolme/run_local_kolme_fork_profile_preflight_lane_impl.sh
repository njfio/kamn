#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-profile-preflight-summary.json"
CHECKOUT_PATH="/tmp/kolme_fork"
PROBE_COMMAND=""
ALLOW_NON_DEFAULT_PROBE_COMMAND="false"
MAX_SECONDS=45

DEFAULT_CHECKOUT_PATH="/tmp/kolme_fork"
DEFAULT_PROBE_PROFILE="example-six-sigma:serve-api-server"
DEFAULT_PROBE_BIN="example-six-sigma"
DEFAULT_PROBE_COMPONENT="api-server"

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
    --checkout-path)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --checkout-path" >&2
        exit 1
      fi
      CHECKOUT_PATH="$2"
      shift 2
      ;;
    --probe-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --probe-command" >&2
        exit 1
      fi
      PROBE_COMMAND="$2"
      shift 2
      ;;
    --allow-non-default-probe-command)
      ALLOW_NON_DEFAULT_PROBE_COMMAND="true"
      shift 1
      ;;
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_profile_preflight_lane.sh [options]

Options:
  --mode dry-run|run                     Emit planned checks or execute local profile preflight.
  --output-json <path>                   Deterministic summary report output path.
  --checkout-path <path>                 Local kolme_fork checkout path.
  --probe-command <command>              Optional probe command override.
  --allow-non-default-probe-command      Allow non-default profile probe command for local harnesses.
  --max-seconds <n>                      Max total runtime budget for run mode.
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

if [ -z "$CHECKOUT_PATH" ]; then
  echo "checkout-path must not be empty" >&2
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

default_probe_command="cd $(printf '%q' "$DEFAULT_CHECKOUT_PATH") && cargo run --bin $DEFAULT_PROBE_BIN -- serve $DEFAULT_PROBE_COMPONENT"
selected_probe_command="$PROBE_COMMAND"
if [ -z "$selected_probe_command" ]; then
  selected_probe_command="$default_probe_command"
fi

if [ -z "$selected_probe_command" ]; then
  echo "probe-command must not be empty" >&2
  exit 1
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  local reason_code="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason_code" >>"$CHECK_FILE"
}

profile_contract_command="default profile: cd ${DEFAULT_CHECKOUT_PATH} && cargo run --bin ${DEFAULT_PROBE_BIN} -- serve ${DEFAULT_PROBE_COMPONENT}"
probe_command_display="$selected_probe_command"
probe_policy_error_message="probe-command must use default cargo profile probe command for /tmp/kolme_fork (or pass --allow-non-default-probe-command for local harnesses)"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0

record_check "profile_contract" "$profile_contract_command" "planned" "not_run"
record_check "probe_command" "$probe_command_display" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "profile_contract" "$profile_contract_command" "fail" "local_opt_in_missing"
    record_check "probe_command" "$probe_command_display" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  elif [ "$ALLOW_NON_DEFAULT_PROBE_COMMAND" != "true" ] && {
    [ "$CHECKOUT_PATH" != "$DEFAULT_CHECKOUT_PATH" ] || \
    [ "$selected_probe_command" != "$default_probe_command" ];
  }; then
    echo "$probe_policy_error_message" >&2
    record_check "profile_contract" "$profile_contract_command" "fail" "command_policy_violation"
    record_check "probe_command" "$probe_command_display" "skipped" "command_policy_violation"
    overall_status="fail"
    reason_code="command_policy_violation"
  else
    record_check "profile_contract" "$profile_contract_command" "pass" "command_profile_validated"

    if [ "$ALLOW_NON_DEFAULT_PROBE_COMMAND" != "true" ]; then
      record_check "probe_command" "$probe_command_display" "pass" "default_probe_profile_validated"
      reason_code="profile_preflight_passed"
    else
      set +e
      timeout "$MAX_SECONDS" bash -lc "$selected_probe_command" >/dev/null 2>&1
      probe_exit_code=$?
      set -e

      if [ "$probe_exit_code" -eq 0 ]; then
        record_check "probe_command" "$probe_command_display" "pass" "probe_command_passed"
        reason_code="profile_preflight_passed"
      elif [ "$probe_exit_code" -eq 124 ]; then
        record_check "probe_command" "$probe_command_display" "fail" "probe_command_timeout"
        overall_status="fail"
        reason_code="checkpoint_failed_probe_command"
      else
        record_check "probe_command" "$probe_command_display" "fail" "probe_command_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_probe_command"
      fi
    fi
  fi

  elapsed_seconds="$(( $(date +%s) - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="preflight_budget_exceeded"
    fi
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_profile_preflight_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECKOUT_PATH" "$selected_probe_command" "$ALLOW_NON_DEFAULT_PROBE_COMMAND" "$CHECK_FILE" "$DEFAULT_CHECKOUT_PATH" "$DEFAULT_PROBE_PROFILE" "$DEFAULT_PROBE_BIN" "$DEFAULT_PROBE_COMPONENT"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
