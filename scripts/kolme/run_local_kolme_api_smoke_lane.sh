#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PROBE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_api_probe_lane.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-api-smoke-summary.json"
PROBE_REPORT="/tmp/kolme-local-api-probe-summary.json"
SMOKE_OUTPUT_FILE="/tmp/kolme-local-api-smoke-output.txt"
BASE_URL="http://127.0.0.1:3000"
SMOKE_COMMAND="curl --silent --show-error --fail http://127.0.0.1:3000/healthz"
FORK_CHAIN_VERSION="v0.15.2"
MAX_SECONDS=60
PROBE_MAX_SECONDS=15

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
    --probe-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --probe-report" >&2
        exit 1
      fi
      PROBE_REPORT="$2"
      shift 2
      ;;
    --smoke-output-file)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --smoke-output-file" >&2
        exit 1
      fi
      SMOKE_OUTPUT_FILE="$2"
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
    --smoke-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --smoke-command" >&2
        exit 1
      fi
      SMOKE_COMMAND="$2"
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
    --max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --max-seconds" >&2
        exit 1
      fi
      MAX_SECONDS="$2"
      shift 2
      ;;
    --probe-max-seconds)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --probe-max-seconds" >&2
        exit 1
      fi
      PROBE_MAX_SECONDS="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_api_smoke_lane.sh [options]

Options:
  --mode dry-run|run              Execute planned output or active smoke lane.
  --output-json <path>            Deterministic summary report output path.
  --probe-report <path>           Local API probe summary path.
  --smoke-output-file <path>      Captured stdout/stderr for smoke command.
  --base-url <url>                Base URL for local Kolme API server.
  --smoke-command <command>       Bounded smoke command to execute.
  --fork-chain-version <value>    Required chain_version query value for probe prerequisite.
  --max-seconds <n>               Max runtime budget for smoke command.
  --probe-max-seconds <n>         Max runtime budget for probe prerequisite.
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

if [ -z "$BASE_URL" ]; then
  echo "base-url must not be empty" >&2
  exit 1
fi

if [ -z "$FORK_CHAIN_VERSION" ]; then
  echo "fork-chain-version must not be empty" >&2
  exit 1
fi

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

if ! [[ "$PROBE_MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$PROBE_MAX_SECONDS" -le 0 ]; then
  echo "probe-max-seconds must be a positive integer" >&2
  exit 1
fi

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

probe_command="bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${PROBE_MAX_SECONDS} --output-json ${PROBE_REPORT}"
record_check "api_probe" "$probe_command" "planned"
record_check "api_smoke_command" "$SMOKE_COMMAND" "planned"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ "${KAMN_KOLME_LOCAL_HEAVY:-0}" != "1" ]; then
    echo "run mode requires explicit local-only opt-in: KAMN_KOLME_LOCAL_HEAVY=1" >&2
    record_check "api_probe" "$probe_command" "fail"
    record_check "api_smoke_command" "$SMOKE_COMMAND" "skipped"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  elif [ ! -x "$PROBE_RUNNER" ]; then
    record_check "api_probe" "$probe_command" "fail"
    record_check "api_smoke_command" "$SMOKE_COMMAND" "skipped"
    overall_status="fail"
    reason_code="probe_runner_missing"
  else
    if bash "$PROBE_RUNNER" \
      --mode run \
      --base-url "$BASE_URL" \
      --fork-chain-version "$FORK_CHAIN_VERSION" \
      --max-seconds "$PROBE_MAX_SECONDS" \
      --output-json "$PROBE_REPORT" >/dev/null; then
      record_check "api_probe" "$probe_command" "pass"
      mkdir -p "$(dirname "$SMOKE_OUTPUT_FILE")"
      smoke_exit_code=0
      set +e
      timeout "${MAX_SECONDS}" bash -lc "$SMOKE_COMMAND" >"$SMOKE_OUTPUT_FILE" 2>&1
      smoke_exit_code=$?
      set -e

      if [ "$smoke_exit_code" -eq 0 ]; then
        record_check "api_smoke_command" "$SMOKE_COMMAND" "pass"
        reason_code="smoke_command_passed"
      elif [ "$smoke_exit_code" -eq 124 ]; then
        record_check "api_smoke_command" "$SMOKE_COMMAND" "fail"
        overall_status="fail"
        reason_code="smoke_command_timeout"
      else
        record_check "api_smoke_command" "$SMOKE_COMMAND" "fail"
        overall_status="fail"
        reason_code="smoke_command_failed"
      fi
    else
      record_check "api_probe" "$probe_command" "fail"
      record_check "api_smoke_command" "$SMOKE_COMMAND" "skipped"
      overall_status="fail"
      reason_code="probe_prerequisite_failed"
    fi
  fi

  end_epoch="$(date +%s)"
  elapsed_seconds="$(( end_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ] && [ "$reason_code" != "smoke_command_timeout" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="smoke_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$local_only_enforced" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$BASE_URL" "$SMOKE_COMMAND" "$FORK_CHAIN_VERSION" "$PROBE_REPORT" "$SMOKE_OUTPUT_FILE" "$CHECK_FILE" <<'PY'
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
base_url = sys.argv[9]
smoke_command = sys.argv[10]
fork_chain_version = sys.argv[11]
probe_report = sys.argv[12]
smoke_output_file = sys.argv[13]
checks_path = pathlib.Path(sys.argv[14])

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
    "schema_version": "kamn.kolme.local-api-smoke-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "base_url": base_url,
    "smoke_command": smoke_command,
    "fork_chain_version": fork_chain_version,
    "probe_report": probe_report,
    "smoke_output_file": smoke_output_file,
    "checks": checks,
    "artifact_paths": [
        probe_report,
        smoke_output_file,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "smoke_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=$local_only_enforced"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
