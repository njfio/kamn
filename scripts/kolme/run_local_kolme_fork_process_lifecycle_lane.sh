#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INTEGRATION_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-process-lifecycle-summary.json"
INTEGRATION_REPORT="/tmp/kolme-local-kamn-live-runtime-integration-summary.json"
PROCESS_OUTPUT_FILE="/tmp/kolme-local-fork-process-lifecycle-process.log"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
SERVE_COMMAND=""
MAX_SECONDS=300
STARTUP_MAX_SECONDS=45
INTEGRATION_MAX_SECONDS=240
INTEGRATION_BOOTSTRAP_MAX_SECONDS=90
INTEGRATION_CONFORMANCE_MAX_SECONDS=180
INTEGRATION_RUNTIME_COMMIT_MAX_SECONDS=30
INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND=""
INTEGRATION_RUNTIME_COMMIT_FINALITY_MAX_SECONDS=15
INTEGRATION_RUNTIME_COMMIT_FINALITY_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-live-finality-output.txt"
PROCESS_PID=""

shell_escape() {
  printf "%q" "$1"
}

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
    --integration-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-report" >&2
        exit 1
      fi
      INTEGRATION_REPORT="$2"
      shift 2
      ;;
    --process-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --process-output-file" >&2
        exit 1
      fi
      PROCESS_OUTPUT_FILE="$2"
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
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --startup-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --startup-max-seconds" >&2
        exit 1
      fi
      STARTUP_MAX_SECONDS="$2"
      shift 2
      ;;
    --integration-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-max-seconds" >&2
        exit 1
      fi
      INTEGRATION_MAX_SECONDS="$2"
      shift 2
      ;;
    --integration-bootstrap-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-bootstrap-max-seconds" >&2
        exit 1
      fi
      INTEGRATION_BOOTSTRAP_MAX_SECONDS="$2"
      shift 2
      ;;
    --integration-conformance-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-conformance-max-seconds" >&2
        exit 1
      fi
      INTEGRATION_CONFORMANCE_MAX_SECONDS="$2"
      shift 2
      ;;
    --integration-runtime-commit-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-runtime-commit-max-seconds" >&2
        exit 1
      fi
      INTEGRATION_RUNTIME_COMMIT_MAX_SECONDS="$2"
      shift 2
      ;;
    --integration-runtime-commit-finality-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-runtime-commit-finality-command" >&2
        exit 1
      fi
      INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND="$2"
      shift 2
      ;;
    --integration-runtime-commit-finality-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-runtime-commit-finality-max-seconds" >&2
        exit 1
      fi
      INTEGRATION_RUNTIME_COMMIT_FINALITY_MAX_SECONDS="$2"
      shift 2
      ;;
    --integration-runtime-commit-finality-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-runtime-commit-finality-output-file" >&2
        exit 1
      fi
      INTEGRATION_RUNTIME_COMMIT_FINALITY_OUTPUT_FILE="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_process_lifecycle_lane.sh [options]

Options:
  --mode dry-run|run                            Emit planned checks or execute lifecycle orchestration.
  --output-json <path>                          Deterministic summary report output path.
  --integration-report <path>                   Output path for local KAMN live runtime integration summary.
  --process-output-file <path>                  Captured stdout/stderr for the lifecycle process command.
  --checkout-path <path>                        Local kolme_fork checkout path.
  --expected-remote-url <url>                   Expected origin URL for checkout validation.
  --expected-ref <ref>                          Expected symbolic HEAD ref for checkout.
  --base-url <url>                              Base URL for local Kolme API server.
  --fork-chain-version <value>                  Required chain_version query value for fork-info checks.
  --serve-command <command>                     Process command used to run local kolme_fork server.
  --max-seconds <n>                             Max total runtime budget for run mode.
  --startup-max-seconds <n>                     Max wait for readiness checks.
  --integration-max-seconds <n>                 Max runtime budget for nested integration lane.
  --integration-bootstrap-max-seconds <n>       Max bootstrap/readiness budget for nested integration lane.
  --integration-conformance-max-seconds <n>     Max live API conformance budget for nested integration lane.
  --integration-runtime-commit-max-seconds <n>  Max runtime-commit endpoint command budget for nested lane.
  --integration-runtime-commit-finality-command <command>
                                                 Optional runtime finality command passed through to nested integration lane.
  --integration-runtime-commit-finality-max-seconds <n>
                                                 Max runtime budget for runtime finality command passed through to nested integration lane.
  --integration-runtime-commit-finality-output-file <path>
                                                 Captured stdout/stderr path for runtime finality command passed through to nested integration lane.
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

if [ -z "$CHECKOUT_PATH" ] || [ -z "$EXPECTED_REMOTE_URL" ] || [ -z "$EXPECTED_REF" ]; then
  echo "checkout-path, expected-remote-url, and expected-ref must not be empty" >&2
  exit 1
fi

if [ -z "$BASE_URL" ] || [ -z "$FORK_CHAIN_VERSION" ]; then
  echo "base-url and fork-chain-version must not be empty" >&2
  exit 1
fi

for numeric_value in \
  "$MAX_SECONDS" \
  "$STARTUP_MAX_SECONDS" \
  "$INTEGRATION_MAX_SECONDS" \
  "$INTEGRATION_BOOTSTRAP_MAX_SECONDS" \
  "$INTEGRATION_CONFORMANCE_MAX_SECONDS" \
  "$INTEGRATION_RUNTIME_COMMIT_MAX_SECONDS" \
  "$INTEGRATION_RUNTIME_COMMIT_FINALITY_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ ! -x "$INTEGRATION_RUNNER" ]; then
  echo "expected local KAMN live runtime integration runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"; if [ -n "$PROCESS_PID" ] && kill -0 "$PROCESS_PID" >/dev/null 2>&1; then kill "$PROCESS_PID" >/dev/null 2>&1 || true; wait "$PROCESS_PID" 2>/dev/null || true; fi' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  local reason_code="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$reason_code" >>"$CHECK_FILE"
}

read_report_reason_code() {
  local report_file="$1"
  python3 - "$report_file" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
if not path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

value = payload.get("reason_code")
if isinstance(value, str) and value.strip():
    print(value)
else:
    print("reason_code_missing")
PY
}

wait_for_readiness() {
  local base_url="$1"
  local chain_version="$2"
  local max_seconds="$3"

  local max_attempts="$(( max_seconds * 10 ))"
  if [ "$max_attempts" -lt 1 ]; then
    max_attempts=1
  fi

  local healthz_url="${base_url%/}/healthz"
  local fork_info_url="${base_url%/}/fork-info?chain_version=${chain_version}"

  for _ in $(seq 1 "$max_attempts"); do
    if [ -n "$PROCESS_PID" ] && ! kill -0 "$PROCESS_PID" >/dev/null 2>&1; then
      return 1
    fi
    if curl --silent --show-error --fail "$healthz_url" >/dev/null 2>&1 \
      && curl --silent --show-error --fail "$fork_info_url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

graceful_teardown() {
  if [ -z "$PROCESS_PID" ]; then
    return 1
  fi

  if ! kill -0 "$PROCESS_PID" >/dev/null 2>&1; then
    return 0
  fi

  kill "$PROCESS_PID" >/dev/null 2>&1 || true
  for _ in $(seq 1 50); do
    if ! kill -0 "$PROCESS_PID" >/dev/null 2>&1; then
      wait "$PROCESS_PID" 2>/dev/null || true
      PROCESS_PID=""
      return 0
    fi
    sleep 0.1
  done

  kill -9 "$PROCESS_PID" >/dev/null 2>&1 || true
  wait "$PROCESS_PID" 2>/dev/null || true
  PROCESS_PID=""
  return 1
}

serve_command_planned="${SERVE_COMMAND:-<required-in-run-mode>}"
readiness_command="curl --silent --show-error --fail ${BASE_URL%/}/healthz && curl --silent --show-error --fail ${BASE_URL%/}/fork-info?chain_version=${FORK_CHAIN_VERSION}"
integration_command="bash scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${INTEGRATION_MAX_SECONDS} --bootstrap-max-seconds ${INTEGRATION_BOOTSTRAP_MAX_SECONDS} --conformance-max-seconds ${INTEGRATION_CONFORMANCE_MAX_SECONDS} --runtime-commit-max-seconds ${INTEGRATION_RUNTIME_COMMIT_MAX_SECONDS} --output-json ${INTEGRATION_REPORT}"
if [ -n "$INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND" ]; then
  integration_command="${integration_command} --runtime-commit-finality-command $(shell_escape "${INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND}") --runtime-commit-finality-max-seconds ${INTEGRATION_RUNTIME_COMMIT_FINALITY_MAX_SECONDS} --runtime-commit-finality-output-file $(shell_escape "${INTEGRATION_RUNTIME_COMMIT_FINALITY_OUTPUT_FILE}")"
fi
teardown_command="kill <lifecycle-process-pid>"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
start_reason_code="not_run"
readiness_reason_code="not_run"
integration_reason_code="not_run"
teardown_reason_code="not_run"

record_check "process_start" "$serve_command_planned" "planned" "not_run"
record_check "readiness_probe" "$readiness_command" "planned" "not_run"
record_check "kamn_live_integration" "$integration_command" "planned" "not_run"
record_check "process_teardown" "$teardown_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "process_start" "$serve_command_planned" "fail" "local_opt_in_missing"
    record_check "readiness_probe" "$readiness_command" "skipped" "local_opt_in_missing"
    record_check "kamn_live_integration" "$integration_command" "skipped" "local_opt_in_missing"
    record_check "process_teardown" "$teardown_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
    start_reason_code="local_opt_in_missing"
    readiness_reason_code="local_opt_in_missing"
    integration_reason_code="local_opt_in_missing"
    teardown_reason_code="local_opt_in_missing"
  elif [ -z "$SERVE_COMMAND" ]; then
    echo "run mode requires --serve-command to start a local kolme_fork process" >&2
    record_check "process_start" "$serve_command_planned" "fail" "serve_command_missing"
    record_check "readiness_probe" "$readiness_command" "skipped" "serve_command_missing"
    record_check "kamn_live_integration" "$integration_command" "skipped" "serve_command_missing"
    record_check "process_teardown" "$teardown_command" "skipped" "serve_command_missing"
    overall_status="fail"
    reason_code="serve_command_missing"
    start_reason_code="serve_command_missing"
    readiness_reason_code="serve_command_missing"
    integration_reason_code="serve_command_missing"
    teardown_reason_code="serve_command_missing"
  else
    mkdir -p "$(dirname "$PROCESS_OUTPUT_FILE")"
    set +e
    bash -lc "$SERVE_COMMAND" >"$PROCESS_OUTPUT_FILE" 2>&1 &
    PROCESS_PID="$!"
    set -e

    sleep 0.2
    if ! kill -0 "$PROCESS_PID" >/dev/null 2>&1; then
      record_check "process_start" "$SERVE_COMMAND" "fail" "process_start_failed"
      record_check "readiness_probe" "$readiness_command" "skipped" "process_start_failed"
      record_check "kamn_live_integration" "$integration_command" "skipped" "process_start_failed"
      record_check "process_teardown" "$teardown_command" "skipped" "process_start_failed"
      overall_status="fail"
      reason_code="process_start_failed"
      start_reason_code="process_start_failed"
      readiness_reason_code="process_start_failed"
      integration_reason_code="process_start_failed"
      teardown_reason_code="process_start_failed"
    else
      record_check "process_start" "$SERVE_COMMAND" "pass" "process_started"
      start_reason_code="process_started"

      if wait_for_readiness "$BASE_URL" "$FORK_CHAIN_VERSION" "$STARTUP_MAX_SECONDS"; then
        record_check "readiness_probe" "$readiness_command" "pass" "readiness_checks_passed"
        readiness_reason_code="readiness_checks_passed"
      else
        record_check "readiness_probe" "$readiness_command" "fail" "process_readiness_failed"
        record_check "kamn_live_integration" "$integration_command" "skipped" "process_readiness_failed"
        overall_status="fail"
        reason_code="process_readiness_failed"
        readiness_reason_code="process_readiness_failed"
        integration_reason_code="process_readiness_failed"
      fi

      if [ "$overall_status" = "ok" ]; then
        integration_exit_code=0
        integration_args=(
          --mode run
          --checkout-path "$CHECKOUT_PATH"
          --expected-remote-url "$EXPECTED_REMOTE_URL"
          --expected-ref "$EXPECTED_REF"
          --base-url "$BASE_URL"
          --fork-chain-version "$FORK_CHAIN_VERSION"
          --max-seconds "$INTEGRATION_MAX_SECONDS"
          --bootstrap-max-seconds "$INTEGRATION_BOOTSTRAP_MAX_SECONDS"
          --conformance-max-seconds "$INTEGRATION_CONFORMANCE_MAX_SECONDS"
          --runtime-commit-max-seconds "$INTEGRATION_RUNTIME_COMMIT_MAX_SECONDS"
          --output-json "$INTEGRATION_REPORT"
        )
        if [ -n "$INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND" ]; then
          integration_args+=(
            --runtime-commit-finality-command "$INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND"
            --runtime-commit-finality-max-seconds "$INTEGRATION_RUNTIME_COMMIT_FINALITY_MAX_SECONDS"
            --runtime-commit-finality-output-file "$INTEGRATION_RUNTIME_COMMIT_FINALITY_OUTPUT_FILE"
          )
        fi
        set +e
        timeout "$INTEGRATION_MAX_SECONDS" \
          env KAMN_KOLME_LOCAL_HEAVY=1 \
          bash "$INTEGRATION_RUNNER" \
            "${integration_args[@]}" >/dev/null
        integration_exit_code=$?
        set -e

        if [ "$integration_exit_code" -eq 0 ]; then
          record_check "kamn_live_integration" "$integration_command" "pass" "kamn_live_integration_passed"
          integration_reason_code="kamn_live_integration_passed"
          reason_code="process_lifecycle_integration_passed"
        elif [ "$integration_exit_code" -eq 124 ]; then
          record_check "kamn_live_integration" "$integration_command" "fail" "kamn_live_integration_timeout"
          overall_status="fail"
          reason_code="kamn_live_integration_failed"
          integration_reason_code="kamn_live_integration_timeout"
        else
          integration_reason_code="$(read_report_reason_code "$INTEGRATION_REPORT")"
          record_check "kamn_live_integration" "$integration_command" "fail" "$integration_reason_code"
          overall_status="fail"
          reason_code="kamn_live_integration_failed"
        fi
      fi

      if graceful_teardown; then
        record_check "process_teardown" "$teardown_command" "pass" "process_teardown_passed"
        teardown_reason_code="process_teardown_passed"
      else
        record_check "process_teardown" "$teardown_command" "fail" "process_teardown_forced"
        teardown_reason_code="process_teardown_forced"
        if [ "$overall_status" = "ok" ]; then
          overall_status="fail"
          reason_code="process_teardown_failed"
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
      reason_code="process_lifecycle_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$BASE_URL" "$FORK_CHAIN_VERSION" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$SERVE_COMMAND" "$PROCESS_OUTPUT_FILE" "$INTEGRATION_REPORT" "$INTEGRATION_RUNTIME_COMMIT_FINALITY_COMMAND" "$INTEGRATION_RUNTIME_COMMIT_FINALITY_MAX_SECONDS" "$INTEGRATION_RUNTIME_COMMIT_FINALITY_OUTPUT_FILE" "$start_reason_code" "$readiness_reason_code" "$integration_reason_code" "$teardown_reason_code" "$CHECK_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
checkout_path = sys.argv[5]
expected_remote_url = sys.argv[6]
expected_ref = sys.argv[7]
base_url = sys.argv[8]
fork_chain_version = sys.argv[9]
elapsed_seconds = int(sys.argv[10])
max_seconds = int(sys.argv[11])
budget_status = sys.argv[12]
serve_command = sys.argv[13]
process_output_file = sys.argv[14]
integration_report = sys.argv[15]
integration_runtime_commit_finality_command = sys.argv[16]
integration_runtime_commit_finality_max_seconds = int(sys.argv[17])
integration_runtime_commit_finality_output_file = sys.argv[18]
start_reason_code = sys.argv[19]
readiness_reason_code = sys.argv[20]
integration_reason_code = sys.argv[21]
teardown_reason_code = sys.argv[22]
checks_path = pathlib.Path(sys.argv[23])

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
    "schema_version": "kamn.kolme.local-fork-process-lifecycle-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "serve_command": serve_command,
    "integration_runtime_commit_finality_enabled": bool(integration_runtime_commit_finality_command),
    "integration_runtime_commit_finality_command": (
        integration_runtime_commit_finality_command if integration_runtime_commit_finality_command else ""
    ),
    "integration_runtime_commit_finality_max_seconds": integration_runtime_commit_finality_max_seconds,
    "integration_runtime_commit_finality_output_file": (
        integration_runtime_commit_finality_output_file if integration_runtime_commit_finality_command else ""
    ),
    "start_reason_code": start_reason_code,
    "readiness_reason_code": readiness_reason_code,
    "integration_reason_code": integration_reason_code,
    "teardown_reason_code": teardown_reason_code,
    "contracts": {
        "healthz_path": "/healthz",
        "fork_info_path": "/fork-info",
        "runtime_commit_endpoint": "/broadcast/runtime-commit",
        "runtime_commit_method": "POST",
        "integration_runner": "run_local_kamn_live_runtime_integration_lane.sh",
    },
    "checks": checks,
    "artifact_paths": [
        process_output_file,
        integration_report,
    ],
}

if integration_runtime_commit_finality_command:
    summary["artifact_paths"].append(integration_runtime_commit_finality_output_file)

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
