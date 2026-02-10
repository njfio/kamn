#!/usr/bin/env bash
set -euo pipefail

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-api-probe-summary.json"
BASE_URL="http://127.0.0.1:3000"
HEALTHZ_PATH="/healthz"
FORK_INFO_PATH="/fork-info"
EXPECTED_HEALTHZ="Healthy!"
MAX_SECONDS=30

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
    --base-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --base-url" >&2
        exit 1
      fi
      BASE_URL="$2"
      shift 2
      ;;
    --healthz-path)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --healthz-path" >&2
        exit 1
      fi
      HEALTHZ_PATH="$2"
      shift 2
      ;;
    --fork-info-path)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-info-path" >&2
        exit 1
      fi
      FORK_INFO_PATH="$2"
      shift 2
      ;;
    --expected-healthz)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-healthz" >&2
        exit 1
      fi
      EXPECTED_HEALTHZ="$2"
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
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_api_probe_lane.sh [options]

Options:
  --mode dry-run|run            Execute planned output or active endpoint checks.
  --output-json <path>          Write deterministic summary JSON to this path.
  --base-url <url>              Base URL for local Kolme API server.
  --healthz-path <path>         Health endpoint path (default: /healthz).
  --fork-info-path <path>       Fork-info endpoint path (default: /fork-info).
  --expected-healthz <text>     Expected health endpoint body.
  --max-seconds <n>             Max runtime budget in seconds for run mode.
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

if [ -z "$HEALTHZ_PATH" ] || [ -z "$FORK_INFO_PATH" ]; then
  echo "endpoint paths must not be empty" >&2
  exit 1
fi

if ! [[ "$MAX_SECONDS" =~ ^[0-9]+$ ]] || [ "$MAX_SECONDS" -le 0 ]; then
  echo "max-seconds must be a positive integer" >&2
  exit 1
fi

CHECK_FILE="$(mktemp)"
HEALTHZ_BODY_FILE="$(mktemp)"
FORK_INFO_BODY_FILE="$(mktemp)"
FORK_INFO_VALUES_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE" "$HEALTHZ_BODY_FILE" "$FORK_INFO_BODY_FILE" "$FORK_INFO_VALUES_FILE"' EXIT

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
fork_first_block=""
fork_last_block=""

healthz_url="${BASE_URL%/}${HEALTHZ_PATH}"
fork_info_url="${BASE_URL%/}${FORK_INFO_PATH}"

record_check "healthz_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${healthz_url}" "planned"
record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "planned"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! command -v curl >/dev/null 2>&1; then
    record_check "healthz_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${healthz_url}" "fail"
    record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "skipped"
    overall_status="fail"
    reason_code="http_client_missing"
  else
    if curl --silent --show-error --max-time "$MAX_SECONDS" "$healthz_url" >"$HEALTHZ_BODY_FILE" 2>/dev/null; then
      observed_healthz="$(cat "$HEALTHZ_BODY_FILE")"
      if [ "$observed_healthz" = "$EXPECTED_HEALTHZ" ]; then
        record_check "healthz_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${healthz_url}" "pass"
      else
        record_check "healthz_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${healthz_url}" "fail"
        record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "skipped"
        overall_status="fail"
        reason_code="healthz_unexpected_body"
      fi
    else
      record_check "healthz_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${healthz_url}" "fail"
      record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "skipped"
      overall_status="fail"
      reason_code="healthz_request_failed"
    fi

    if [ "$overall_status" = "ok" ]; then
      if curl --silent --show-error --max-time "$MAX_SECONDS" "$fork_info_url" >"$FORK_INFO_BODY_FILE" 2>/dev/null; then
        if python3 - "$FORK_INFO_BODY_FILE" "$FORK_INFO_VALUES_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

body_path = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])

try:
    payload = json.loads(body_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as error:
    raise SystemExit(f"invalid json: {error.msg}") from error

if not isinstance(payload, dict):
    raise SystemExit("fork-info payload must be a JSON object")

first_block = payload.get("first_block")
last_block = payload.get("last_block")
if not isinstance(first_block, int) or not isinstance(last_block, int):
    raise SystemExit("fork-info payload must include integer first_block and last_block")

out_path.write_text(
    f"first_block={first_block}\nlast_block={last_block}\n",
    encoding="utf-8",
)
PY
        then
          # shellcheck disable=SC1090
          . "$FORK_INFO_VALUES_FILE"
          fork_first_block="$first_block"
          fork_last_block="$last_block"
          record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "pass"
          reason_code="probe_checks_passed"
        else
          record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "fail"
          overall_status="fail"
          reason_code="fork_info_invalid_payload"
        fi
      else
        record_check "fork_info_endpoint" "curl --silent --show-error --max-time ${MAX_SECONDS} ${fork_info_url}" "fail"
        overall_status="fail"
        reason_code="fork_info_request_failed"
      fi
    fi
  fi

  end_epoch="$(date +%s)"
  elapsed_seconds="$(( end_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="probe_budget_exceeded"
    fi
  fi
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$BASE_URL" "$HEALTHZ_PATH" "$FORK_INFO_PATH" "$EXPECTED_HEALTHZ" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$fork_first_block" "$fork_last_block" "$CHECK_FILE" <<'PY'
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
base_url = sys.argv[5]
healthz_path = sys.argv[6]
fork_info_path = sys.argv[7]
expected_healthz = sys.argv[8]
elapsed_seconds = int(sys.argv[9])
max_seconds = int(sys.argv[10])
budget_status = sys.argv[11]
fork_first_block_raw = sys.argv[12]
fork_last_block_raw = sys.argv[13]
checks_path = pathlib.Path(sys.argv[14])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 3:
        continue
    check_id, command, check_status = parts
    checks.append({"id": check_id, "command": command, "status": check_status})

fork_info = {
    "first_block": int(fork_first_block_raw) if fork_first_block_raw else None,
    "last_block": int(fork_last_block_raw) if fork_last_block_raw else None,
}

summary = {
    "schema_version": "kamn.kolme.local-api-probe-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "base_url": base_url,
    "healthz_path": healthz_path,
    "fork_info_path": fork_info_path,
    "expected_healthz": expected_healthz,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "fork_info": fork_info,
    "checks": checks,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "probe_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
