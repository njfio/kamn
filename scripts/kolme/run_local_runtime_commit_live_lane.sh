#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-runtime-commit-live-summary.json"
LIVE_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-live-output.txt"
LIVE_COMMAND=""
FINALITY_COMMAND=""
MAX_SECONDS=90
BASE_URL="http://127.0.0.1:3000"
PROVIDER_HINT="kolme-fork-local"
AUTHORIZATION_HEADER=""
PREFLIGHT_MAX_SECONDS=10
FINALITY_MAX_SECONDS=15
SKIP_PREFLIGHT=0
FINALITY_OUTPUT_FILE="/tmp/kolme-local-runtime-commit-live-finality-output.txt"

shell_escape() {
  printf "%q" "$1"
}

default_live_command() {
  local command
  command="KAMN_KOLME_LIVE_BASE_URL=$(shell_escape "$BASE_URL")"
  command="${command} KAMN_KOLME_LIVE_PROVIDER_HINT=$(shell_escape "$PROVIDER_HINT")"
  if [ -n "$AUTHORIZATION_HEADER" ]; then
    command="${command} KAMN_KOLME_LIVE_AUTHORIZATION=$(shell_escape "$AUTHORIZATION_HEADER")"
  fi
  command="${command} cargo test -p kamn-core --test kolme_runtime_commit_http_transport -- --ignored --exact integration_kolme_fork_live_node_submit_reaches_endpoint"
  printf '%s' "$command"
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
    --live-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --live-output-file" >&2
        exit 1
      fi
      LIVE_OUTPUT_FILE="$2"
      shift 2
      ;;
    --live-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --live-command" >&2
        exit 1
      fi
      LIVE_COMMAND="$2"
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
    --finality-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-command" >&2
        exit 1
      fi
      FINALITY_COMMAND="$2"
      shift 2
      ;;
    --finality-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-max-seconds" >&2
        exit 1
      fi
      FINALITY_MAX_SECONDS="$2"
      shift 2
      ;;
    --finality-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --finality-output-file" >&2
        exit 1
      fi
      FINALITY_OUTPUT_FILE="$2"
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
    --provider-hint)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --provider-hint" >&2
        exit 1
      fi
      PROVIDER_HINT="$2"
      shift 2
      ;;
    --authorization-header)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --authorization-header" >&2
        exit 1
      fi
      AUTHORIZATION_HEADER="$2"
      shift 2
      ;;
    --preflight-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --preflight-max-seconds" >&2
        exit 1
      fi
      PREFLIGHT_MAX_SECONDS="$2"
      shift 2
      ;;
    --skip-preflight)
      SKIP_PREFLIGHT=1
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_runtime_commit_live_lane.sh [options]

Options:
  --mode dry-run|run            Execute planned output or active local live lane.
  --output-json <path>          Deterministic summary report output path.
  --live-output-file <path>     Captured stdout/stderr for live command.
  --live-command <command>      Runtime-commit submit/finality command for run mode.
  --finality-command <command>  Optional post-submit finality command for run mode.
  --finality-output-file <path> Captured stdout/stderr for finality command.
  --finality-max-seconds <n>    Max runtime budget in seconds for finality command.
  --max-seconds <n>             Max runtime budget in seconds for run mode.
  --base-url <url>              Live Kolme base URL used by default smoke command.
  --provider-hint <value>       Provider hint used by default live smoke command.
  --authorization-header <str>  Optional Authorization header value for live smoke.
  --preflight-max-seconds <n>   Max runtime budget for preflight health probe.
  --skip-preflight              Bypass preflight health probe in run mode.
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

if ! [[ "$PREFLIGHT_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$PREFLIGHT_MAX_SECONDS" -le 0 ]; then
  echo "preflight-max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$FINALITY_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$FINALITY_MAX_SECONDS" -le 0 ]; then
  echo "finality-max-seconds must be a positive integer" >&2
  exit 1
fi

if [ -z "$BASE_URL" ]; then
  echo "base-url must not be empty" >&2
  exit 1
fi

if [ -z "$PROVIDER_HINT" ]; then
  echo "provider-hint must not be empty" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ -z "$LIVE_COMMAND" ]; then
  LIVE_COMMAND="$(default_live_command)"
fi

preflight_command="curl --silent --show-error --fail --max-time ${PREFLIGHT_MAX_SECONDS} ${BASE_URL%/}/healthz"

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  printf '%s\t%s\t%s\n' "$check_id" "$command" "$status" >>"$CHECK_FILE"
}

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
local_only_enforced="true"

planned_command="$LIVE_COMMAND"
planned_finality_command="$FINALITY_COMMAND"
if [ -z "$planned_finality_command" ]; then
  planned_finality_command="<not-configured>"
fi
record_check "runtime_commit_live_preflight" "$preflight_command" "planned"
record_check "runtime_commit_live_command" "$planned_command" "planned"
record_check "runtime_commit_live_finality_command" "$planned_finality_command" "planned"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "runtime_commit_live_command" "$planned_command" "fail"
    record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  elif [ "$SKIP_PREFLIGHT" -eq 1 ]; then
    record_check "runtime_commit_live_preflight" "$preflight_command" "skipped"
  else
    set +e
    timeout "${PREFLIGHT_MAX_SECONDS}" bash -lc "$preflight_command" >/dev/null 2>&1
    preflight_exit_code=$?
    set -e

    if [ "$preflight_exit_code" -eq 0 ]; then
      record_check "runtime_commit_live_preflight" "$preflight_command" "pass"
    elif [ "$preflight_exit_code" -eq 124 ]; then
      record_check "runtime_commit_live_preflight" "$preflight_command" "fail"
      overall_status="fail"
      reason_code="live_preflight_timeout"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    else
      record_check "runtime_commit_live_preflight" "$preflight_command" "fail"
      overall_status="fail"
      reason_code="live_preflight_failed"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    mkdir -p "$(dirname "$LIVE_OUTPUT_FILE")"
    command_exit_code=0
    set +e
    timeout "${MAX_SECONDS}" bash -lc "$LIVE_COMMAND" >"$LIVE_OUTPUT_FILE" 2>&1
    command_exit_code=$?
    set -e

    if [ "$command_exit_code" -eq 0 ]; then
      record_check "runtime_commit_live_command" "$LIVE_COMMAND" "pass"
      reason_code="live_runtime_commit_command_passed"
    elif [ "$command_exit_code" -eq 124 ]; then
      record_check "runtime_commit_live_command" "$LIVE_COMMAND" "fail"
      overall_status="fail"
      reason_code="live_runtime_commit_command_timeout"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    else
      record_check "runtime_commit_live_command" "$LIVE_COMMAND" "fail"
      overall_status="fail"
      reason_code="live_runtime_commit_command_failed"
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    if [ -n "$FINALITY_COMMAND" ]; then
      mkdir -p "$(dirname "$FINALITY_OUTPUT_FILE")"
      finality_exit_code=0
      set +e
      timeout "${FINALITY_MAX_SECONDS}" bash -lc "$FINALITY_COMMAND" >"$FINALITY_OUTPUT_FILE" 2>&1
      finality_exit_code=$?
      set -e

      if [ "$finality_exit_code" -eq 0 ]; then
        record_check "runtime_commit_live_finality_command" "$FINALITY_COMMAND" "pass"
        reason_code="live_runtime_commit_and_finality_commands_passed"
      elif [ "$finality_exit_code" -eq 124 ]; then
        record_check "runtime_commit_live_finality_command" "$FINALITY_COMMAND" "fail"
        overall_status="fail"
        reason_code="live_finality_command_timeout"
      else
        record_check "runtime_commit_live_finality_command" "$FINALITY_COMMAND" "fail"
        overall_status="fail"
        reason_code="live_finality_command_failed"
      fi
    else
      record_check "runtime_commit_live_finality_command" "$planned_finality_command" "skipped"
    fi
  fi

  end_epoch="$(date +%s)"
  elapsed_seconds="$(( end_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ] && [ "$reason_code" != "live_runtime_commit_command_timeout" ] && [ "$reason_code" != "live_finality_command_timeout" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="live_runtime_commit_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$local_only_enforced" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$LIVE_COMMAND" "$LIVE_OUTPUT_FILE" "$FINALITY_COMMAND" "$FINALITY_OUTPUT_FILE" "$BASE_URL" "$PROVIDER_HINT" "$PREFLIGHT_MAX_SECONDS" "$FINALITY_MAX_SECONDS" "$SKIP_PREFLIGHT" "$CHECK_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
local_only_enforced = sys.argv[5] == "true"
elapsed_seconds = int(sys.argv[6])
max_seconds = int(sys.argv[7])
budget_status = sys.argv[8]
live_command = sys.argv[9]
live_output_file = sys.argv[10]
finality_command = sys.argv[11]
finality_output_file = sys.argv[12]
base_url = sys.argv[13]
provider_hint = sys.argv[14]
preflight_max_seconds = int(sys.argv[15])
finality_max_seconds = int(sys.argv[16])
skip_preflight = sys.argv[17] == "1"
checks_path = pathlib.Path(sys.argv[18])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 3:
        continue
    check_id, command, check_status = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-runtime-commit-live-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "live_command": live_command,
    "live_output_file": live_output_file,
    "finality_command": finality_command,
    "finality_output_file": finality_output_file,
    "finality_enabled": bool(finality_command.strip()),
    "finality_max_seconds": finality_max_seconds,
    "base_url": base_url,
    "provider_hint": provider_hint,
    "preflight_enabled": not skip_preflight,
    "preflight_max_seconds": preflight_max_seconds,
    "checks": checks,
    "artifact_paths": [
        live_output_file,
        finality_output_file,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=$local_only_enforced"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
