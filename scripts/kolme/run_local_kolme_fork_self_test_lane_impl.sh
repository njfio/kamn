#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh"
MATRIX_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-self-test-summary.json"
MATRIX_REPORT="/tmp/kolme-local-fork-rust-test-matrix-summary.json"
MATRIX_POLICY_REPORT="/tmp/kolme-local-fork-rust-test-matrix-policy.json"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
MAX_SECONDS=120
MATRIX_MAX_SECONDS=60
MATRIX_CARGO_PROFILE="portable"

declare -a MATRIX_COMMANDS=()

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
    --matrix-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-report" >&2
        exit 1
      fi
      MATRIX_REPORT="$2"
      shift 2
      ;;
    --matrix-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-policy-report" >&2
        exit 1
      fi
      MATRIX_POLICY_REPORT="$2"
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
    --expected-remote-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-remote-url" >&2
        exit 1
      fi
      EXPECTED_REMOTE_URL="$2"
      shift 2
      ;;
    --expected-ref)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-ref" >&2
        exit 1
      fi
      EXPECTED_REF="$2"
      shift 2
      ;;
    --matrix-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-command" >&2
        exit 1
      fi
      MATRIX_COMMANDS+=("$2")
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
    --matrix-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-max-seconds" >&2
        exit 1
      fi
      MATRIX_MAX_SECONDS="$2"
      shift 2
      ;;
    --matrix-cargo-profile)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-cargo-profile" >&2
        exit 1
      fi
      MATRIX_CARGO_PROFILE="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_self_test_lane.sh [options]

Options:
  --mode dry-run|run                    Emit planned checks or execute local self-test orchestration.
  --output-json <path>                  Deterministic summary output path.
  --matrix-report <path>                Output path for nested rust test matrix summary.
  --matrix-policy-report <path>         Output path for nested rust test matrix policy report.
  --checkout-path <path>                Local kolme_fork checkout path.
  --expected-remote-url <url>           Expected origin URL for checkout validation.
  --expected-ref <ref>                  Expected symbolic HEAD ref for checkout.
  --matrix-command <command>            Repeatable override commands for nested matrix execution.
  --max-seconds <n>                     Max total runtime budget for this self-test lane.
  --matrix-max-seconds <n>              Max runtime budget for nested matrix lane execution.
  --matrix-cargo-profile <strict|portable>
                                         Cargo command execution profile passed to nested matrix lane.
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

for numeric_value in "$MAX_SECONDS" "$MATRIX_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ "$MATRIX_CARGO_PROFILE" != "strict" ] && [ "$MATRIX_CARGO_PROFILE" != "portable" ]; then
  echo "matrix-cargo-profile must be one of: strict, portable" >&2
  exit 1
fi

if [ -z "$CHECKOUT_PATH" ] || [ -z "$EXPECTED_REMOTE_URL" ] || [ -z "$EXPECTED_REF" ]; then
  echo "checkout-path, expected-remote-url, and expected-ref must not be empty" >&2
  exit 1
fi

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected local fork rust test matrix runner to be executable" >&2
  exit 1
fi

if [ ! -x "$MATRIX_CHECKER" ]; then
  echo "expected local fork rust test matrix policy checker to be executable" >&2
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
  local reason_code="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason_code" >>"$CHECK_FILE"
}

read_reason_code() {
  local report_file="$1"
  python3 "$ROOT_DIR/scripts/kolme/contracts/read_json_field_or_default.py" "$report_file" "reason_code" "reason_code_missing"
}

matrix_command="bash scripts/kolme/run_local_kolme_fork_rust_test_matrix_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --max-seconds ${MATRIX_MAX_SECONDS} --cargo-profile ${MATRIX_CARGO_PROFILE} --output-json ${MATRIX_REPORT}"
policy_command="python3 scripts/kolme/check_local_kolme_fork_rust_test_matrix_policy.py --report-file ${MATRIX_REPORT} --expected-final-decision GO --ci-fast-gate PASS --require-reason-code fork_rust_test_matrix_passed --output-json ${MATRIX_POLICY_REPORT}"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0

record_check "self_test_matrix_lane" "$matrix_command" "planned" "not_run"
record_check "self_test_matrix_policy" "$policy_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "self_test_matrix_lane" "$matrix_command" "fail" "local_opt_in_missing"
    record_check "self_test_matrix_policy" "$policy_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  else
    matrix_args=(
      --mode run
      --checkout-path "$CHECKOUT_PATH"
      --expected-remote-url "$EXPECTED_REMOTE_URL"
      --expected-ref "$EXPECTED_REF"
      --max-seconds "$MATRIX_MAX_SECONDS"
      --cargo-profile "$MATRIX_CARGO_PROFILE"
      --output-json "$MATRIX_REPORT"
    )
    for matrix_command_override in "${MATRIX_COMMANDS[@]}"; do
      matrix_args+=(--matrix-command "$matrix_command_override")
    done

    set +e
    timeout "$MATRIX_MAX_SECONDS" bash "$MATRIX_RUNNER" "${matrix_args[@]}" >/dev/null 2>&1
    matrix_exit_code=$?
    set -e

    if [ "$matrix_exit_code" -eq 0 ]; then
      record_check "self_test_matrix_lane" "$matrix_command" "pass" "self_test_matrix_lane_passed"
    elif [ "$matrix_exit_code" -eq 124 ]; then
      record_check "self_test_matrix_lane" "$matrix_command" "fail" "self_test_matrix_lane_timeout"
      record_check "self_test_matrix_policy" "$policy_command" "skipped" "self_test_matrix_lane_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_self_test_matrix_lane"
    else
      matrix_reason_code="$(read_reason_code "$MATRIX_REPORT")"
      record_check "self_test_matrix_lane" "$matrix_command" "fail" "$matrix_reason_code"
      record_check "self_test_matrix_policy" "$policy_command" "skipped" "self_test_matrix_lane_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_self_test_matrix_lane"
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      python3 "$MATRIX_CHECKER" \
        --report-file "$MATRIX_REPORT" \
        --expected-final-decision GO \
        --ci-fast-gate PASS \
        --require-reason-code fork_rust_test_matrix_passed \
        --output-json "$MATRIX_POLICY_REPORT" >/dev/null 2>&1
      matrix_policy_exit_code=$?
      set -e

      if [ "$matrix_policy_exit_code" -eq 0 ]; then
        record_check "self_test_matrix_policy" "$policy_command" "pass" "self_test_matrix_policy_passed"
        reason_code="fork_self_test_passed"
      else
        record_check "self_test_matrix_policy" "$policy_command" "fail" "self_test_matrix_policy_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_self_test_matrix_policy"
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
      reason_code="self_test_budget_exceeded"
    fi
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_self_test_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$MATRIX_MAX_SECONDS" "$budget_status" "$CHECK_FILE" "$MATRIX_REPORT" "$MATRIX_POLICY_REPORT" "$MATRIX_CARGO_PROFILE"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
