#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROFILE_PREFLIGHT_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh"
PROFILE_PREFLIGHT_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py"
SELF_TEST_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_self_test_lane.sh"
SELF_TEST_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_self_test_policy.py"
LIFECYCLE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh"
LIFECYCLE_CHECKER="$ROOT_DIR/scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-real-process-summary.json"
PROFILE_PREFLIGHT_REPORT="/tmp/kolme-local-fork-profile-preflight-summary.json"
PROFILE_PREFLIGHT_POLICY_REPORT="/tmp/kolme-local-fork-profile-preflight-policy.json"
SELF_TEST_REPORT="/tmp/kolme-local-fork-self-test-summary.json"
SELF_TEST_POLICY_REPORT="/tmp/kolme-local-fork-self-test-policy.json"
LIFECYCLE_REPORT="/tmp/kolme-local-fork-process-lifecycle-summary.json"
LIFECYCLE_POLICY_REPORT="/tmp/kolme-local-fork-process-lifecycle-policy.json"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
SERVE_COMMAND=""
ALLOW_NON_FORK_SERVE_COMMAND="false"
MAX_SECONDS=360
PROFILE_PREFLIGHT_MAX_SECONDS=45
SELF_TEST_MAX_SECONDS=120
SELF_TEST_MATRIX_MAX_SECONDS=60
LIFECYCLE_MAX_SECONDS=300
LIFECYCLE_STARTUP_MAX_SECONDS=45
LIFECYCLE_INTEGRATION_MAX_SECONDS=240
LIFECYCLE_BOOTSTRAP_MAX_SECONDS=90
LIFECYCLE_CONFORMANCE_MAX_SECONDS=180
LIFECYCLE_RUNTIME_COMMIT_MAX_SECONDS=30

declare -a SELF_TEST_MATRIX_COMMANDS=()

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
    --lifecycle-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-report" >&2
        exit 1
      fi
      LIFECYCLE_REPORT="$2"
      shift 2
      ;;
    --preflight-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --preflight-report" >&2
        exit 1
      fi
      PROFILE_PREFLIGHT_REPORT="$2"
      shift 2
      ;;
    --preflight-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --preflight-policy-report" >&2
        exit 1
      fi
      PROFILE_PREFLIGHT_POLICY_REPORT="$2"
      shift 2
      ;;
    --self-test-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --self-test-report" >&2
        exit 1
      fi
      SELF_TEST_REPORT="$2"
      shift 2
      ;;
    --self-test-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --self-test-policy-report" >&2
        exit 1
      fi
      SELF_TEST_POLICY_REPORT="$2"
      shift 2
      ;;
    --lifecycle-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-policy-report" >&2
        exit 1
      fi
      LIFECYCLE_POLICY_REPORT="$2"
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
    --base-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --base-url" >&2
        exit 1
      fi
      BASE_URL="$2"
      shift 2
      ;;
    --fork-chain-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-chain-version" >&2
        exit 1
      fi
      FORK_CHAIN_VERSION="$2"
      shift 2
      ;;
    --serve-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --serve-command" >&2
        exit 1
      fi
      SERVE_COMMAND="$2"
      shift 2
      ;;
    --allow-non-fork-serve-command)
      ALLOW_NON_FORK_SERVE_COMMAND="true"
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
    --preflight-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --preflight-max-seconds" >&2
        exit 1
      fi
      PROFILE_PREFLIGHT_MAX_SECONDS="$2"
      shift 2
      ;;
    --self-test-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --self-test-max-seconds" >&2
        exit 1
      fi
      SELF_TEST_MAX_SECONDS="$2"
      shift 2
      ;;
    --self-test-matrix-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --self-test-matrix-max-seconds" >&2
        exit 1
      fi
      SELF_TEST_MATRIX_MAX_SECONDS="$2"
      shift 2
      ;;
    --self-test-matrix-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --self-test-matrix-command" >&2
        exit 1
      fi
      SELF_TEST_MATRIX_COMMANDS+=("$2")
      shift 2
      ;;
    --lifecycle-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-max-seconds" >&2
        exit 1
      fi
      LIFECYCLE_MAX_SECONDS="$2"
      shift 2
      ;;
    --lifecycle-startup-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-startup-max-seconds" >&2
        exit 1
      fi
      LIFECYCLE_STARTUP_MAX_SECONDS="$2"
      shift 2
      ;;
    --lifecycle-integration-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-integration-max-seconds" >&2
        exit 1
      fi
      LIFECYCLE_INTEGRATION_MAX_SECONDS="$2"
      shift 2
      ;;
    --lifecycle-bootstrap-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-bootstrap-max-seconds" >&2
        exit 1
      fi
      LIFECYCLE_BOOTSTRAP_MAX_SECONDS="$2"
      shift 2
      ;;
    --lifecycle-conformance-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-conformance-max-seconds" >&2
        exit 1
      fi
      LIFECYCLE_CONFORMANCE_MAX_SECONDS="$2"
      shift 2
      ;;
    --lifecycle-runtime-commit-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --lifecycle-runtime-commit-max-seconds" >&2
        exit 1
      fi
      LIFECYCLE_RUNTIME_COMMIT_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_real_process_contract_lane.sh [options]

Options:
  --mode dry-run|run                              Emit planned checks or execute wrapper checks.
  --output-json <path>                            Deterministic summary output path.
  --preflight-report <path>                       Output path for local fork profile preflight report.
  --preflight-policy-report <path>                Output path for local fork profile preflight policy report.
  --self-test-report <path>                       Output path for local fork self-test summary report.
  --self-test-policy-report <path>                Output path for local fork self-test policy report.
  --lifecycle-report <path>                       Output path for local fork process lifecycle report.
  --lifecycle-policy-report <path>                Output path for local fork process lifecycle policy report.
  --checkout-path <path>                          Local kolme_fork checkout path.
  --expected-remote-url <url>                     Expected origin URL for checkout validation.
  --expected-ref <ref>                            Expected symbolic HEAD ref for checkout.
  --base-url <url>                                Base URL for local Kolme API server.
  --fork-chain-version <value>                    Required chain_version query value for fork-info checks.
  --serve-command <command>                       Optional serve command override.
  --allow-non-fork-serve-command                  Allow non-fork serve command override for local test harnesses.
  --max-seconds <n>                               Max total runtime budget for run mode.
  --preflight-max-seconds <n>                     Max runtime budget for nested profile-preflight lane.
  --self-test-max-seconds <n>                     Max runtime budget for nested self-test lane.
  --self-test-matrix-max-seconds <n>              Max per-command budget for nested self-test matrix runner.
  --self-test-matrix-command <command>            Repeatable override command passed to nested self-test lane.
  --lifecycle-max-seconds <n>                     Max runtime budget for nested process-lifecycle lane.
  --lifecycle-startup-max-seconds <n>             Max startup readiness budget for nested lane.
  --lifecycle-integration-max-seconds <n>         Max integration budget for nested lane.
  --lifecycle-bootstrap-max-seconds <n>           Max bootstrap budget for nested lane.
  --lifecycle-conformance-max-seconds <n>         Max conformance budget for nested lane.
  --lifecycle-runtime-commit-max-seconds <n>      Max runtime-commit budget for nested lane.
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

for numeric_value in \
  "$MAX_SECONDS" \
  "$PROFILE_PREFLIGHT_MAX_SECONDS" \
  "$SELF_TEST_MAX_SECONDS" \
  "$SELF_TEST_MATRIX_MAX_SECONDS" \
  "$LIFECYCLE_MAX_SECONDS" \
  "$LIFECYCLE_STARTUP_MAX_SECONDS" \
  "$LIFECYCLE_INTEGRATION_MAX_SECONDS" \
  "$LIFECYCLE_BOOTSTRAP_MAX_SECONDS" \
  "$LIFECYCLE_CONFORMANCE_MAX_SECONDS" \
  "$LIFECYCLE_RUNTIME_COMMIT_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ ! -x "$PROFILE_PREFLIGHT_RUNNER" ]; then
  echo "expected local fork profile preflight runner to be executable" >&2
  exit 1
fi

if [ ! -x "$PROFILE_PREFLIGHT_CHECKER" ]; then
  echo "expected local fork profile preflight policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$SELF_TEST_RUNNER" ]; then
  echo "expected local fork self-test runner to be executable" >&2
  exit 1
fi

if [ ! -x "$SELF_TEST_CHECKER" ]; then
  echo "expected local fork self-test policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LIFECYCLE_RUNNER" ]; then
  echo "expected local fork process lifecycle runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LIFECYCLE_CHECKER" ]; then
  echo "expected local fork process lifecycle policy checker to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

default_serve_command="cd $(printf '%q' "$CHECKOUT_PATH") && cargo run --bin example-six-sigma -- serve api-server"
selected_serve_command="$SERVE_COMMAND"
if [ -z "$selected_serve_command" ]; then
  selected_serve_command="$default_serve_command"
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
  python3 - "$report_file" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
if not report_path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(report_path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

reason_code = payload.get("reason_code")
if isinstance(reason_code, str) and reason_code.strip():
    print(reason_code)
else:
    print("reason_code_missing")
PY
}

command_policy_error_message="serve-command must target checkout path and use cargo run for the local fork profile (or pass --allow-non-fork-serve-command for local harnesses)"

command_profile_contract="default profile: cargo run --bin example-six-sigma -- serve api-server"
preflight_command="bash scripts/kolme/run_local_kolme_fork_profile_preflight_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --max-seconds ${PROFILE_PREFLIGHT_MAX_SECONDS} --output-json ${PROFILE_PREFLIGHT_REPORT}"
preflight_policy_command="python3 scripts/kolme/check_local_kolme_fork_profile_preflight_policy.py --report-file ${PROFILE_PREFLIGHT_REPORT} --expected-final-decision GO --ci-fast-gate PASS --require-reason-code profile_preflight_passed --output-json ${PROFILE_PREFLIGHT_POLICY_REPORT}"
self_test_command="bash scripts/kolme/run_local_kolme_fork_self_test_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --max-seconds ${SELF_TEST_MAX_SECONDS} --matrix-max-seconds ${SELF_TEST_MATRIX_MAX_SECONDS} --output-json ${SELF_TEST_REPORT}"
self_test_policy_command="python3 scripts/kolme/check_local_kolme_fork_self_test_policy.py --report-file ${SELF_TEST_REPORT} --expected-final-decision GO --ci-fast-gate PASS --require-reason-code fork_self_test_passed --output-json ${SELF_TEST_POLICY_REPORT}"
lifecycle_command="bash scripts/kolme/run_local_kolme_fork_process_lifecycle_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --serve-command ${selected_serve_command} --max-seconds ${LIFECYCLE_MAX_SECONDS} --startup-max-seconds ${LIFECYCLE_STARTUP_MAX_SECONDS} --integration-max-seconds ${LIFECYCLE_INTEGRATION_MAX_SECONDS} --integration-bootstrap-max-seconds ${LIFECYCLE_BOOTSTRAP_MAX_SECONDS} --integration-conformance-max-seconds ${LIFECYCLE_CONFORMANCE_MAX_SECONDS} --integration-runtime-commit-max-seconds ${LIFECYCLE_RUNTIME_COMMIT_MAX_SECONDS} --output-json ${LIFECYCLE_REPORT}"
policy_command="python3 scripts/kolme/check_local_kolme_fork_process_lifecycle_policy.py --report-file ${LIFECYCLE_REPORT} --expected-final-decision GO --ci-fast-gate PASS --require-reason-code process_lifecycle_integration_passed --output-json ${LIFECYCLE_POLICY_REPORT}"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0

record_check "real_fork_command_profile" "$command_profile_contract" "planned" "not_run"
record_check "profile_preflight_lane" "$preflight_command" "planned" "not_run"
record_check "profile_preflight_policy" "$preflight_policy_command" "planned" "not_run"
record_check "self_test_lane" "$self_test_command" "planned" "not_run"
record_check "self_test_policy" "$self_test_policy_command" "planned" "not_run"
record_check "process_lifecycle_lane" "$lifecycle_command" "planned" "not_run"
record_check "process_lifecycle_policy" "$policy_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "real_fork_command_profile" "$command_profile_contract" "fail" "local_opt_in_missing"
    record_check "profile_preflight_lane" "$preflight_command" "skipped" "local_opt_in_missing"
    record_check "profile_preflight_policy" "$preflight_policy_command" "skipped" "local_opt_in_missing"
    record_check "self_test_lane" "$self_test_command" "skipped" "local_opt_in_missing"
    record_check "self_test_policy" "$self_test_policy_command" "skipped" "local_opt_in_missing"
    record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "local_opt_in_missing"
    record_check "process_lifecycle_policy" "$policy_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  elif [ "$ALLOW_NON_FORK_SERVE_COMMAND" != "true" ] && {
    [[ "$selected_serve_command" != *"$CHECKOUT_PATH"* ]] || \
    [[ "$selected_serve_command" != *"cargo run"* ]] || \
    [[ "$selected_serve_command" != *"example-six-sigma"* ]] || \
    [[ "$selected_serve_command" != *"serve api-server"* ]];
  }; then
    echo "$command_policy_error_message" >&2
    record_check "real_fork_command_profile" "$command_profile_contract" "fail" "command_policy_violation"
    record_check "profile_preflight_lane" "$preflight_command" "skipped" "command_policy_violation"
    record_check "profile_preflight_policy" "$preflight_policy_command" "skipped" "command_policy_violation"
    record_check "self_test_lane" "$self_test_command" "skipped" "command_policy_violation"
    record_check "self_test_policy" "$self_test_policy_command" "skipped" "command_policy_violation"
    record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "command_policy_violation"
    record_check "process_lifecycle_policy" "$policy_command" "skipped" "command_policy_violation"
    overall_status="fail"
    reason_code="command_policy_violation"
  else
    record_check "real_fork_command_profile" "$command_profile_contract" "pass" "command_profile_validated"

    set +e
    preflight_args=(
      --mode run
      --checkout-path "$CHECKOUT_PATH"
      --max-seconds "$PROFILE_PREFLIGHT_MAX_SECONDS"
      --output-json "$PROFILE_PREFLIGHT_REPORT"
    )
    if [ "$ALLOW_NON_FORK_SERVE_COMMAND" = "true" ]; then
      preflight_args+=(--probe-command "bash -lc 'exit 0'" --allow-non-default-probe-command)
    fi
    timeout "$PROFILE_PREFLIGHT_MAX_SECONDS" bash "$PROFILE_PREFLIGHT_RUNNER" "${preflight_args[@]}" >/dev/null 2>&1
    preflight_exit_code=$?
    set -e

    if [ "$preflight_exit_code" -eq 0 ]; then
      record_check "profile_preflight_lane" "$preflight_command" "pass" "profile_preflight_lane_passed"
    elif [ "$preflight_exit_code" -eq 124 ]; then
      record_check "profile_preflight_lane" "$preflight_command" "fail" "profile_preflight_lane_timeout"
      record_check "profile_preflight_policy" "$preflight_policy_command" "skipped" "profile_preflight_lane_failed"
      record_check "self_test_lane" "$self_test_command" "skipped" "profile_preflight_lane_failed"
      record_check "self_test_policy" "$self_test_policy_command" "skipped" "profile_preflight_lane_failed"
      record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "profile_preflight_lane_failed"
      record_check "process_lifecycle_policy" "$policy_command" "skipped" "profile_preflight_lane_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_profile_preflight_lane"
    else
      preflight_reason_code="$(read_reason_code "$PROFILE_PREFLIGHT_REPORT")"
      record_check "profile_preflight_lane" "$preflight_command" "fail" "$preflight_reason_code"
      record_check "profile_preflight_policy" "$preflight_policy_command" "skipped" "profile_preflight_lane_failed"
      record_check "self_test_lane" "$self_test_command" "skipped" "profile_preflight_lane_failed"
      record_check "self_test_policy" "$self_test_policy_command" "skipped" "profile_preflight_lane_failed"
      record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "profile_preflight_lane_failed"
      record_check "process_lifecycle_policy" "$policy_command" "skipped" "profile_preflight_lane_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_profile_preflight_lane"
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      python3 "$PROFILE_PREFLIGHT_CHECKER" \
        --report-file "$PROFILE_PREFLIGHT_REPORT" \
        --expected-final-decision GO \
        --ci-fast-gate PASS \
        --require-reason-code profile_preflight_passed \
        --output-json "$PROFILE_PREFLIGHT_POLICY_REPORT" >/dev/null 2>&1
      preflight_policy_exit_code=$?
      set -e

      if [ "$preflight_policy_exit_code" -eq 0 ]; then
        record_check "profile_preflight_policy" "$preflight_policy_command" "pass" "profile_preflight_policy_passed"
      else
        record_check "profile_preflight_policy" "$preflight_policy_command" "fail" "profile_preflight_policy_failed"
        record_check "self_test_lane" "$self_test_command" "skipped" "profile_preflight_policy_failed"
        record_check "self_test_policy" "$self_test_policy_command" "skipped" "profile_preflight_policy_failed"
        record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "profile_preflight_policy_failed"
        record_check "process_lifecycle_policy" "$policy_command" "skipped" "profile_preflight_policy_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_profile_preflight_policy"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      self_test_args=(
        --mode run
        --checkout-path "$CHECKOUT_PATH"
        --expected-remote-url "$EXPECTED_REMOTE_URL"
        --expected-ref "$EXPECTED_REF"
        --max-seconds "$SELF_TEST_MAX_SECONDS"
        --matrix-max-seconds "$SELF_TEST_MATRIX_MAX_SECONDS"
        --output-json "$SELF_TEST_REPORT"
      )
      for self_test_matrix_command in "${SELF_TEST_MATRIX_COMMANDS[@]}"; do
        self_test_args+=(--matrix-command "$self_test_matrix_command")
      done

      set +e
      timeout "$SELF_TEST_MAX_SECONDS" bash "$SELF_TEST_RUNNER" "${self_test_args[@]}" >/dev/null 2>&1
      self_test_exit_code=$?
      set -e

      if [ "$self_test_exit_code" -eq 0 ]; then
        record_check "self_test_lane" "$self_test_command" "pass" "self_test_lane_passed"
      elif [ "$self_test_exit_code" -eq 124 ]; then
        record_check "self_test_lane" "$self_test_command" "fail" "self_test_lane_timeout"
        record_check "self_test_policy" "$self_test_policy_command" "skipped" "self_test_lane_failed"
        record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "self_test_lane_failed"
        record_check "process_lifecycle_policy" "$policy_command" "skipped" "self_test_lane_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_self_test_lane"
      else
        self_test_reason_code="$(read_reason_code "$SELF_TEST_REPORT")"
        record_check "self_test_lane" "$self_test_command" "fail" "$self_test_reason_code"
        record_check "self_test_policy" "$self_test_policy_command" "skipped" "self_test_lane_failed"
        record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "self_test_lane_failed"
        record_check "process_lifecycle_policy" "$policy_command" "skipped" "self_test_lane_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_self_test_lane"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      python3 "$SELF_TEST_CHECKER" \
        --report-file "$SELF_TEST_REPORT" \
        --expected-final-decision GO \
        --ci-fast-gate PASS \
        --require-reason-code fork_self_test_passed \
        --output-json "$SELF_TEST_POLICY_REPORT" >/dev/null 2>&1
      self_test_policy_exit_code=$?
      set -e

      if [ "$self_test_policy_exit_code" -eq 0 ]; then
        record_check "self_test_policy" "$self_test_policy_command" "pass" "self_test_policy_passed"
      else
        record_check "self_test_policy" "$self_test_policy_command" "fail" "self_test_policy_failed"
        record_check "process_lifecycle_lane" "$lifecycle_command" "skipped" "self_test_policy_failed"
        record_check "process_lifecycle_policy" "$policy_command" "skipped" "self_test_policy_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_self_test_policy"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout "$LIFECYCLE_MAX_SECONDS" bash "$LIFECYCLE_RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "$EXPECTED_REMOTE_URL" \
      --expected-ref "$EXPECTED_REF" \
      --base-url "$BASE_URL" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --serve-command "$selected_serve_command" \
      --max-seconds "$LIFECYCLE_MAX_SECONDS" \
      --startup-max-seconds "$LIFECYCLE_STARTUP_MAX_SECONDS" \
      --integration-max-seconds "$LIFECYCLE_INTEGRATION_MAX_SECONDS" \
      --integration-bootstrap-max-seconds "$LIFECYCLE_BOOTSTRAP_MAX_SECONDS" \
      --integration-conformance-max-seconds "$LIFECYCLE_CONFORMANCE_MAX_SECONDS" \
      --integration-runtime-commit-max-seconds "$LIFECYCLE_RUNTIME_COMMIT_MAX_SECONDS" \
      --output-json "$LIFECYCLE_REPORT" >/dev/null 2>&1
      lifecycle_exit_code=$?
      set -e

      if [ "$lifecycle_exit_code" -eq 0 ]; then
        record_check "process_lifecycle_lane" "$lifecycle_command" "pass" "process_lifecycle_lane_passed"
      elif [ "$lifecycle_exit_code" -eq 124 ]; then
        record_check "process_lifecycle_lane" "$lifecycle_command" "fail" "process_lifecycle_lane_timeout"
        record_check "process_lifecycle_policy" "$policy_command" "skipped" "process_lifecycle_lane_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_process_lifecycle_lane"
      else
        lifecycle_reason_code="$(read_reason_code "$LIFECYCLE_REPORT")"
        record_check "process_lifecycle_lane" "$lifecycle_command" "fail" "$lifecycle_reason_code"
        record_check "process_lifecycle_policy" "$policy_command" "skipped" "process_lifecycle_lane_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_process_lifecycle_lane"
      fi

      if [ "$overall_status" = "ok" ]; then
        set +e
        python3 "$LIFECYCLE_CHECKER" \
          --report-file "$LIFECYCLE_REPORT" \
          --expected-final-decision GO \
          --ci-fast-gate PASS \
          --require-reason-code process_lifecycle_integration_passed \
          --output-json "$LIFECYCLE_POLICY_REPORT" >/dev/null 2>&1
        lifecycle_policy_exit_code=$?
        set -e

        if [ "$lifecycle_policy_exit_code" -eq 0 ]; then
          record_check "process_lifecycle_policy" "$policy_command" "pass" "process_lifecycle_policy_passed"
          reason_code="real_fork_process_wrapper_passed"
        else
          record_check "process_lifecycle_policy" "$policy_command" "fail" "process_lifecycle_policy_failed"
          overall_status="fail"
          reason_code="checkpoint_failed_process_lifecycle_policy"
        fi
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
      reason_code="wrapper_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECK_FILE" "$selected_serve_command" "$ALLOW_NON_FORK_SERVE_COMMAND" "$PROFILE_PREFLIGHT_REPORT" "$PROFILE_PREFLIGHT_POLICY_REPORT" "$SELF_TEST_REPORT" "$SELF_TEST_POLICY_REPORT" "$LIFECYCLE_REPORT" "$LIFECYCLE_POLICY_REPORT" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
budget_status = sys.argv[7]
checks_path = pathlib.Path(sys.argv[8])
selected_serve_command = sys.argv[9]
allow_non_fork_serve_command = sys.argv[10] == "true"
profile_preflight_report = sys.argv[11]
profile_preflight_policy_report = sys.argv[12]
self_test_report = sys.argv[13]
self_test_policy_report = sys.argv[14]
lifecycle_report = sys.argv[15]
lifecycle_policy_report = sys.argv[16]

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
    "schema_version": "kamn.kolme.local-fork-real-process-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "selected_serve_command": selected_serve_command,
    "allow_non_fork_serve_command": allow_non_fork_serve_command,
    "contracts": {
        "default_profile": "example-six-sigma:serve-api-server",
        "expected_cargo_bin": "example-six-sigma",
        "expected_component": "api-server",
        "profile_preflight_runner": "run_local_kolme_fork_profile_preflight_lane.sh",
        "profile_preflight_checker": "check_local_kolme_fork_profile_preflight_policy.py",
        "self_test_runner": "run_local_kolme_fork_self_test_lane.sh",
        "self_test_checker": "check_local_kolme_fork_self_test_policy.py",
        "lifecycle_runner": "run_local_kolme_fork_process_lifecycle_lane.sh",
        "lifecycle_checker": "check_local_kolme_fork_process_lifecycle_policy.py",
    },
    "checks": checks,
    "artifact_paths": [
        profile_preflight_report,
        profile_preflight_policy_report,
        self_test_report,
        self_test_policy_report,
        lifecycle_report,
        lifecycle_policy_report,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
