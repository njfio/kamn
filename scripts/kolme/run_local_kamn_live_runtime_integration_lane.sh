#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BOOTSTRAP_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh"
CONFORMANCE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_live_api_conformance_harness.sh"
LOCALHOST_SIGNED_INTEGRATION_RUNNER="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-kamn-live-runtime-integration-summary.json"
BOOTSTRAP_REPORT="/tmp/kolme-local-fork-bootstrap-readiness-summary.json"
CONFORMANCE_REPORT="/tmp/kolme-local-live-api-conformance-summary.json"
LOCALHOST_SIGNED_REPORT="/tmp/localhost-signed-integration-contract-report.json"
RUNTIME_COMMIT_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-endpoint-output.txt"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
RUNTIME_COMMIT_COMMAND=""
MAX_SECONDS=210
BOOTSTRAP_MAX_SECONDS=90
CONFORMANCE_MAX_SECONDS=180
LOCALHOST_SIGNED_MAX_SECONDS=45
RUNTIME_COMMIT_MAX_SECONDS=30

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
    --bootstrap-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --bootstrap-report" >&2
        exit 1
      fi
      BOOTSTRAP_REPORT="$2"
      shift 2
      ;;
    --conformance-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --conformance-report" >&2
        exit 1
      fi
      CONFORMANCE_REPORT="$2"
      shift 2
      ;;
    --localhost-signed-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-report" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_REPORT="$2"
      shift 2
      ;;
    --runtime-commit-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-output-file" >&2
        exit 1
      fi
      RUNTIME_COMMIT_OUTPUT_FILE="$2"
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
    --runtime-commit-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-command" >&2
        exit 1
      fi
      RUNTIME_COMMIT_COMMAND="$2"
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
    --bootstrap-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --bootstrap-max-seconds" >&2
        exit 1
      fi
      BOOTSTRAP_MAX_SECONDS="$2"
      shift 2
      ;;
    --conformance-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --conformance-max-seconds" >&2
        exit 1
      fi
      CONFORMANCE_MAX_SECONDS="$2"
      shift 2
      ;;
    --localhost-signed-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-max-seconds" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_MAX_SECONDS="$2"
      shift 2
      ;;
    --runtime-commit-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --runtime-commit-max-seconds" >&2
        exit 1
      fi
      RUNTIME_COMMIT_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kamn_live_runtime_integration_lane.sh [options]

Options:
  --mode dry-run|run                    Emit planned checks or execute local live checks.
  --output-json <path>                  Deterministic summary report output path.
  --bootstrap-report <path>             Output path for bootstrap/readiness summary.
  --conformance-report <path>           Output path for live API conformance summary.
  --localhost-signed-report <path>      Output path for localhost signed integration summary.
  --runtime-commit-output-file <path>   Captured stdout/stderr for runtime-commit endpoint command.
  --checkout-path <path>                Local kolme_fork checkout path.
  --expected-remote-url <url>           Expected origin URL for checkout validation.
  --expected-ref <ref>                  Expected symbolic HEAD ref for checkout.
  --base-url <url>                      Base URL for local Kolme API server.
  --fork-chain-version <value>          Required chain_version query value for fork-info checks.
  --runtime-commit-command <command>    Runtime-commit endpoint command for run mode.
  --max-seconds <n>                     Max total runtime budget for run mode.
  --bootstrap-max-seconds <n>           Max budget for bootstrap/readiness prerequisite.
  --conformance-max-seconds <n>         Max budget for live API conformance prerequisite.
  --localhost-signed-max-seconds <n>    Max budget for localhost signed integration prerequisite.
  --runtime-commit-max-seconds <n>      Max budget for runtime-commit endpoint command.
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

for numeric_value in "$MAX_SECONDS" "$BOOTSTRAP_MAX_SECONDS" "$CONFORMANCE_MAX_SECONDS" "$LOCALHOST_SIGNED_MAX_SECONDS" "$RUNTIME_COMMIT_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ ! -x "$BOOTSTRAP_RUNNER" ]; then
  echo "expected local fork bootstrap/readiness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CONFORMANCE_RUNNER" ]; then
  echo "expected local Kolme live API conformance harness runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCALHOST_SIGNED_INTEGRATION_RUNNER" ]; then
  echo "expected localhost signed integration contract runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

default_runtime_commit_command="curl --silent --show-error --fail --request POST --header \"Content-Type: application/json\" --data '{\"commit_id\":\"local-runtime-commit\"}' ${BASE_URL%/}/broadcast/runtime-commit"
if [ -z "$RUNTIME_COMMIT_COMMAND" ]; then
  RUNTIME_COMMIT_COMMAND="$default_runtime_commit_command"
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

bootstrap_command="bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${BOOTSTRAP_MAX_SECONDS} --probe-max-seconds 20 --output-json ${BOOTSTRAP_REPORT}"
localhost_signed_command="bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json ${LOCALHOST_SIGNED_REPORT}"
conformance_command="bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${CONFORMANCE_MAX_SECONDS} --probe-max-seconds 30 --native-max-seconds 120 --output-json ${CONFORMANCE_REPORT}"
runtime_commit_command="$RUNTIME_COMMIT_COMMAND"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
bootstrap_reason_code="not_run"
localhost_signed_reason_code="not_run"
conformance_reason_code="not_run"
runtime_commit_reason_code="not_run"

record_check "bootstrap_readiness" "$bootstrap_command" "planned" "not_run"
record_check "localhost_signed_integration" "$localhost_signed_command" "planned" "not_run"
record_check "live_api_conformance" "$conformance_command" "planned" "not_run"
record_check "runtime_commit_endpoint" "$runtime_commit_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "bootstrap_readiness" "$bootstrap_command" "fail" "local_opt_in_missing"
    record_check "localhost_signed_integration" "$localhost_signed_command" "skipped" "local_opt_in_missing"
    record_check "live_api_conformance" "$conformance_command" "skipped" "local_opt_in_missing"
    record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
    bootstrap_reason_code="local_opt_in_missing"
    localhost_signed_reason_code="local_opt_in_missing"
    conformance_reason_code="local_opt_in_missing"
    runtime_commit_reason_code="local_opt_in_missing"
  else
    if KAMN_KOLME_LOCAL_HEAVY=1 bash "$BOOTSTRAP_RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "$EXPECTED_REMOTE_URL" \
      --expected-ref "$EXPECTED_REF" \
      --base-url "$BASE_URL" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --max-seconds "$BOOTSTRAP_MAX_SECONDS" \
      --probe-max-seconds 20 \
      --output-json "$BOOTSTRAP_REPORT" >/dev/null; then
      record_check "bootstrap_readiness" "$bootstrap_command" "pass" "bootstrap_readiness_passed"
      bootstrap_reason_code="bootstrap_readiness_passed"
    else
      bootstrap_reason_code="$(read_report_reason_code "$BOOTSTRAP_REPORT")"
      record_check "bootstrap_readiness" "$bootstrap_command" "fail" "$bootstrap_reason_code"
      record_check "localhost_signed_integration" "$localhost_signed_command" "skipped" "bootstrap_readiness_failed"
      record_check "live_api_conformance" "$conformance_command" "skipped" "bootstrap_readiness_failed"
      record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "bootstrap_readiness_failed"
      overall_status="fail"
      reason_code="bootstrap_readiness_failed"
      localhost_signed_reason_code="bootstrap_readiness_failed"
      conformance_reason_code="bootstrap_readiness_failed"
      runtime_commit_reason_code="bootstrap_readiness_failed"
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout "$LOCALHOST_SIGNED_MAX_SECONDS" bash "$LOCALHOST_SIGNED_INTEGRATION_RUNNER" \
        --output-json "$LOCALHOST_SIGNED_REPORT" >/dev/null 2>&1
      localhost_signed_exit_code=$?
      set -e

      if [ "$localhost_signed_exit_code" -eq 0 ]; then
        record_check "localhost_signed_integration" "$localhost_signed_command" "pass" "localhost_signed_integration_passed"
        localhost_signed_reason_code="localhost_signed_integration_passed"
      elif [ "$localhost_signed_exit_code" -eq 124 ]; then
        record_check "localhost_signed_integration" "$localhost_signed_command" "fail" "localhost_signed_integration_timeout"
        record_check "live_api_conformance" "$conformance_command" "skipped" "localhost_signed_integration_failed"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "localhost_signed_integration_failed"
        overall_status="fail"
        reason_code="localhost_signed_integration_failed"
        localhost_signed_reason_code="localhost_signed_integration_timeout"
        conformance_reason_code="localhost_signed_integration_failed"
        runtime_commit_reason_code="localhost_signed_integration_failed"
      else
        record_check "localhost_signed_integration" "$localhost_signed_command" "fail" "localhost_signed_integration_failed"
        record_check "live_api_conformance" "$conformance_command" "skipped" "localhost_signed_integration_failed"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "localhost_signed_integration_failed"
        overall_status="fail"
        reason_code="localhost_signed_integration_failed"
        localhost_signed_reason_code="localhost_signed_integration_failed"
        conformance_reason_code="localhost_signed_integration_failed"
        runtime_commit_reason_code="localhost_signed_integration_failed"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      if KAMN_KOLME_LOCAL_HEAVY=1 bash "$CONFORMANCE_RUNNER" \
        --mode run \
        --base-url "$BASE_URL" \
        --fork-chain-version "$FORK_CHAIN_VERSION" \
        --max-seconds "$CONFORMANCE_MAX_SECONDS" \
        --probe-max-seconds 30 \
        --native-max-seconds 120 \
        --output-json "$CONFORMANCE_REPORT" >/dev/null; then
        record_check "live_api_conformance" "$conformance_command" "pass" "live_api_conformance_passed"
        conformance_reason_code="live_api_conformance_passed"
      else
        conformance_reason_code="$(read_report_reason_code "$CONFORMANCE_REPORT")"
        record_check "live_api_conformance" "$conformance_command" "fail" "$conformance_reason_code"
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "skipped" "live_api_conformance_failed"
        overall_status="fail"
        reason_code="live_api_conformance_failed"
        runtime_commit_reason_code="live_api_conformance_failed"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      mkdir -p "$(dirname "$RUNTIME_COMMIT_OUTPUT_FILE")"
      runtime_commit_exit_code=0
      set +e
      timeout "$RUNTIME_COMMIT_MAX_SECONDS" bash -lc "$runtime_commit_command" >"$RUNTIME_COMMIT_OUTPUT_FILE" 2>&1
      runtime_commit_exit_code=$?
      set -e

      if [ "$runtime_commit_exit_code" -eq 0 ]; then
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "pass" "runtime_commit_endpoint_passed"
        runtime_commit_reason_code="runtime_commit_endpoint_passed"
        reason_code="live_runtime_integration_passed"
      elif [ "$runtime_commit_exit_code" -eq 124 ]; then
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "fail" "runtime_commit_endpoint_timeout"
        runtime_commit_reason_code="runtime_commit_endpoint_timeout"
        overall_status="fail"
        reason_code="runtime_commit_endpoint_failed"
      else
        record_check "runtime_commit_endpoint" "$runtime_commit_command" "fail" "runtime_commit_endpoint_failed"
        runtime_commit_reason_code="runtime_commit_endpoint_failed"
        overall_status="fail"
        reason_code="runtime_commit_endpoint_failed"
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
      reason_code="runtime_integration_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$BASE_URL" "$FORK_CHAIN_VERSION" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$runtime_commit_command" "$RUNTIME_COMMIT_OUTPUT_FILE" "$BOOTSTRAP_REPORT" "$LOCALHOST_SIGNED_REPORT" "$CONFORMANCE_REPORT" "$bootstrap_reason_code" "$localhost_signed_reason_code" "$conformance_reason_code" "$runtime_commit_reason_code" "$CHECK_FILE" <<'PY'
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
runtime_commit_command = sys.argv[13]
runtime_commit_output_file = sys.argv[14]
bootstrap_report = sys.argv[15]
localhost_signed_report = sys.argv[16]
conformance_report = sys.argv[17]
bootstrap_reason_code = sys.argv[18]
localhost_signed_reason_code = sys.argv[19]
conformance_reason_code = sys.argv[20]
runtime_commit_reason_code = sys.argv[21]
checks_path = pathlib.Path(sys.argv[22])

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
    "schema_version": "kamn.kolme.local-kamn-live-runtime-integration-summary.v1",
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
    "runtime_commit_command": runtime_commit_command,
    "bootstrap_reason_code": bootstrap_reason_code,
    "localhost_signed_reason_code": localhost_signed_reason_code,
    "conformance_reason_code": conformance_reason_code,
    "runtime_commit_reason_code": runtime_commit_reason_code,
    "contracts": {
        "runtime_commit_endpoint": "/broadcast/runtime-commit",
        "runtime_commit_method": "POST",
        "runtime_commit_finality_primary_endpoint": "/notifications",
        "runtime_commit_finality_fallback_endpoint": "/block/{height}",
    },
    "checks": checks,
    "artifact_paths": [
        bootstrap_report,
        localhost_signed_report,
        conformance_report,
        runtime_commit_output_file,
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
