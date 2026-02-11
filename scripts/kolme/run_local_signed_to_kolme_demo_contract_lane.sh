#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SIGNED_DEMO_RUNNER="$ROOT_DIR/scripts/sdk/run_localhost_signed_demo_contract_lane.sh"
SIGNED_INTEGRATION_RUNNER="$ROOT_DIR/scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
KOLME_RUNTIME_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-signed-to-kolme-demo-summary.json"
LOCALHOST_SIGNED_DEMO_REPORT="/tmp/localhost-signed-demo-contract-report.json"
LOCALHOST_SIGNED_INTEGRATION_REPORT="/tmp/localhost-signed-integration-contract-report.json"
KOLME_RUNTIME_REPORT="/tmp/kolme-local-kamn-live-runtime-integration-summary.json"
KOLME_RUNTIME_POLICY_REPORT="/tmp/kolme-local-kamn-live-runtime-integration-policy.json"
MAX_SECONDS=420
LOCALHOST_SIGNED_DEMO_MAX_SECONDS=60
LOCALHOST_SIGNED_INTEGRATION_MAX_SECONDS=120
KOLME_RUNTIME_INTEGRATION_MAX_SECONDS=300

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
    --localhost-signed-demo-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-demo-report" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_DEMO_REPORT="$2"
      shift 2
      ;;
    --localhost-signed-integration-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-integration-report" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_INTEGRATION_REPORT="$2"
      shift 2
      ;;
    --kolme-runtime-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --kolme-runtime-report" >&2
        exit 1
      fi
      KOLME_RUNTIME_REPORT="$2"
      shift 2
      ;;
    --kolme-runtime-policy-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --kolme-runtime-policy-report" >&2
        exit 1
      fi
      KOLME_RUNTIME_POLICY_REPORT="$2"
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
    --localhost-signed-demo-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-demo-max-seconds" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_DEMO_MAX_SECONDS="$2"
      shift 2
      ;;
    --localhost-signed-integration-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --localhost-signed-integration-max-seconds" >&2
        exit 1
      fi
      LOCALHOST_SIGNED_INTEGRATION_MAX_SECONDS="$2"
      shift 2
      ;;
    --kolme-runtime-integration-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --kolme-runtime-integration-max-seconds" >&2
        exit 1
      fi
      KOLME_RUNTIME_INTEGRATION_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_signed_to_kolme_demo_contract_lane.sh [options]

Options:
  --mode dry-run|run                               Emit planned checks or execute local demo checks.
  --output-json <path>                             Deterministic summary report output path.
  --localhost-signed-demo-report <path>            Output path for localhost signed demo contract report.
  --localhost-signed-integration-report <path>     Output path for localhost signed integration contract report.
  --kolme-runtime-report <path>                    Output path for local KAMN runtime integration report.
  --kolme-runtime-policy-report <path>             Output path for local KAMN runtime integration policy report.
  --max-seconds <n>                                Max total runtime budget for run mode.
  --localhost-signed-demo-max-seconds <n>          Max budget for localhost signed demo stage.
  --localhost-signed-integration-max-seconds <n>   Max budget for localhost signed integration stage.
  --kolme-runtime-integration-max-seconds <n>      Max budget for local KAMN runtime integration stage.
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

for numeric_value in "$MAX_SECONDS" "$LOCALHOST_SIGNED_DEMO_MAX_SECONDS" "$LOCALHOST_SIGNED_INTEGRATION_MAX_SECONDS" "$KOLME_RUNTIME_INTEGRATION_MAX_SECONDS"; do
  if ! [[ "$numeric_value" =~ ^[0-9]+$ ]] || [ "$numeric_value" -le 0 ]; then
    echo "all max-second arguments must be positive integers" >&2
    exit 1
  fi
done

if [ ! -x "$SIGNED_DEMO_RUNNER" ]; then
  echo "expected localhost signed demo contract runner to be executable" >&2
  exit 1
fi

if [ ! -x "$SIGNED_INTEGRATION_RUNNER" ]; then
  echo "expected localhost signed integration contract runner to be executable" >&2
  exit 1
fi

if [ ! -x "$KOLME_RUNTIME_RUNNER" ]; then
  echo "expected local KAMN runtime integration contract runner to be executable" >&2
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

localhost_signed_demo_command="bash scripts/sdk/run_localhost_signed_demo_contract_lane.sh --output-json ${LOCALHOST_SIGNED_DEMO_REPORT}"
localhost_signed_integration_command="bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json ${LOCALHOST_SIGNED_INTEGRATION_REPORT}"
kolme_runtime_integration_command="bash scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh --output-json ${KOLME_RUNTIME_REPORT} --policy-output-json ${KOLME_RUNTIME_POLICY_REPORT}"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0

record_check "localhost_signed_demo_contract" "$localhost_signed_demo_command" "planned" "not_run"
record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "planned" "not_run"
record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "localhost_signed_demo_contract" "$localhost_signed_demo_command" "fail" "local_opt_in_missing"
    record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "skipped" "local_opt_in_missing"
    record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  else
    set +e
    timeout "$LOCALHOST_SIGNED_DEMO_MAX_SECONDS" bash "$SIGNED_DEMO_RUNNER" \
      --output-json "$LOCALHOST_SIGNED_DEMO_REPORT" >/dev/null 2>&1
    signed_demo_exit_code=$?
    set -e

    if [ "$signed_demo_exit_code" -eq 0 ]; then
      record_check "localhost_signed_demo_contract" "$localhost_signed_demo_command" "pass" "localhost_signed_demo_passed"
    elif [ "$signed_demo_exit_code" -eq 124 ]; then
      record_check "localhost_signed_demo_contract" "$localhost_signed_demo_command" "fail" "localhost_signed_demo_timeout"
      record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "skipped" "localhost_signed_demo_failed"
      record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "skipped" "localhost_signed_demo_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_localhost_signed_demo_contract"
    else
      record_check "localhost_signed_demo_contract" "$localhost_signed_demo_command" "fail" "localhost_signed_demo_failed"
      record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "skipped" "localhost_signed_demo_failed"
      record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "skipped" "localhost_signed_demo_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_localhost_signed_demo_contract"
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout "$LOCALHOST_SIGNED_INTEGRATION_MAX_SECONDS" bash "$SIGNED_INTEGRATION_RUNNER" \
        --output-json "$LOCALHOST_SIGNED_INTEGRATION_REPORT" >/dev/null 2>&1
      signed_integration_exit_code=$?
      set -e

      if [ "$signed_integration_exit_code" -eq 0 ]; then
        record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "pass" "localhost_signed_integration_passed"
      elif [ "$signed_integration_exit_code" -eq 124 ]; then
        record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "fail" "localhost_signed_integration_timeout"
        record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "skipped" "localhost_signed_integration_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_localhost_signed_integration_contract"
      else
        record_check "localhost_signed_integration_contract" "$localhost_signed_integration_command" "fail" "localhost_signed_integration_failed"
        record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "skipped" "localhost_signed_integration_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_localhost_signed_integration_contract"
      fi
    fi

    if [ "$overall_status" = "ok" ]; then
      set +e
      timeout "$KOLME_RUNTIME_INTEGRATION_MAX_SECONDS" bash "$KOLME_RUNTIME_RUNNER" \
        --output-json "$KOLME_RUNTIME_REPORT" \
        --policy-output-json "$KOLME_RUNTIME_POLICY_REPORT" >/dev/null 2>&1
      kolme_runtime_exit_code=$?
      set -e

      if [ "$kolme_runtime_exit_code" -eq 0 ]; then
        record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "pass" "local_kamn_runtime_integration_passed"
        reason_code="signed_to_kolme_demo_passed"
      elif [ "$kolme_runtime_exit_code" -eq 124 ]; then
        record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "fail" "local_kamn_runtime_integration_timeout"
        overall_status="fail"
        reason_code="checkpoint_failed_local_kamn_runtime_integration_contract"
      else
        record_check "local_kamn_runtime_integration_contract" "$kolme_runtime_integration_command" "fail" "local_kamn_runtime_integration_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_local_kamn_runtime_integration_contract"
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
      reason_code="demo_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECK_FILE" "$LOCALHOST_SIGNED_DEMO_REPORT" "$LOCALHOST_SIGNED_INTEGRATION_REPORT" "$KOLME_RUNTIME_REPORT" "$KOLME_RUNTIME_POLICY_REPORT" <<'PY'
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
localhost_signed_demo_report = sys.argv[9]
localhost_signed_integration_report = sys.argv[10]
kolme_runtime_report = sys.argv[11]
kolme_runtime_policy_report = sys.argv[12]

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
    "schema_version": "kamn.kolme.local-signed-to-kolme-demo-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checks": checks,
    "artifact_paths": [
        localhost_signed_demo_report,
        localhost_signed_integration_report,
        kolme_runtime_report,
        kolme_runtime_policy_report,
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
