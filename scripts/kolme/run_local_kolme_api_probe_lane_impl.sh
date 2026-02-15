#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-api-probe-summary.json"
BASE_URL="http://127.0.0.1:3000"
HEALTHZ_PATH="/healthz"
FORK_INFO_PATH="/fork-info"
FORK_CHAIN_VERSION="v0.15.2"
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
    --fork-chain-version)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-chain-version" >&2
        exit 1
      fi
      FORK_CHAIN_VERSION="$2"
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
  --fork-chain-version <value>  Required chain_version query value for fork-info checks.
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

if [ -z "$FORK_CHAIN_VERSION" ]; then
  echo "fork-chain-version must not be empty" >&2
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
fork_info_separator="?"
if [[ "$FORK_INFO_PATH" == *"?"* ]]; then
  fork_info_separator="&"
fi
fork_info_url="${BASE_URL%/}${FORK_INFO_PATH}${fork_info_separator}chain_version=${FORK_CHAIN_VERSION}"

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
        if python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_api_probe_fork_info_parse.py" "$FORK_INFO_BODY_FILE" "$FORK_INFO_VALUES_FILE"
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

python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_api_probe_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$BASE_URL" "$HEALTHZ_PATH" "$FORK_INFO_PATH" "$FORK_CHAIN_VERSION" "$EXPECTED_HEALTHZ" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$fork_first_block" "$fork_last_block" "$CHECK_FILE"

echo "status=$overall_status"
echo "probe_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
