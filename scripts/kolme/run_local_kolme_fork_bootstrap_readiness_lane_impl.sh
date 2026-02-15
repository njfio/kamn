#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SYNC_RUNNER="$ROOT_DIR/scripts/kolme/run_local_fork_sync_metadata_lane.sh"
PROBE_RUNNER="$ROOT_DIR/scripts/kolme/run_local_kolme_api_probe_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-bootstrap-readiness-summary.json"
SYNC_REPORT="/tmp/kolme-local-fork-sync-metadata-summary.json"
PROBE_REPORT="/tmp/kolme-local-api-probe-summary.json"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
BASE_URL="http://127.0.0.1:3000"
FORK_CHAIN_VERSION="v0.15.2"
MAX_SECONDS=90
PROBE_MAX_SECONDS=20

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
    --sync-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --sync-report" >&2
        exit 1
      fi
      SYNC_REPORT="$2"
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
Usage: run_local_kolme_fork_bootstrap_readiness_lane.sh [options]

Options:
  --mode dry-run|run              Emit planned checks or execute bootstrap/readiness checks.
  --output-json <path>            Deterministic summary report output path.
  --sync-report <path>            Output path for fork sync metadata summary.
  --probe-report <path>           Output path for local API probe summary.
  --checkout-path <path>          Local kolme_fork checkout path.
  --expected-remote-url <url>     Expected origin URL for checkout validation.
  --expected-ref <ref>            Expected symbolic HEAD ref for checkout.
  --base-url <url>                Base URL for local Kolme API server.
  --fork-chain-version <value>    Required chain_version query value for fork-info checks.
  --max-seconds <n>               Max total runtime budget for run mode.
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

if [ -z "$CHECKOUT_PATH" ] || [ -z "$EXPECTED_REMOTE_URL" ] || [ -z "$EXPECTED_REF" ]; then
  echo "checkout-path, expected-remote-url, and expected-ref must not be empty" >&2
  exit 1
fi

if [ -z "$BASE_URL" ] || [ -z "$FORK_CHAIN_VERSION" ]; then
  echo "base-url and fork-chain-version must not be empty" >&2
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

if [ ! -x "$SYNC_RUNNER" ]; then
  echo "expected local fork sync metadata runner to be executable" >&2
  exit 1
fi

if [ ! -x "$PROBE_RUNNER" ]; then
  echo "expected local Kolme API probe runner to be executable" >&2
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

read_report_reason_code() {
  local report_file="$1"
  python3 "$ROOT_DIR/scripts/kolme/contracts/read_json_field_or_default.py" "$report_file" "reason_code" "reason_code_missing"
}

sync_command="bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path ${CHECKOUT_PATH} --expected-remote-url ${EXPECTED_REMOTE_URL} --expected-ref ${EXPECTED_REF} --output-json ${SYNC_REPORT}"
probe_command="bash scripts/kolme/run_local_kolme_api_probe_lane.sh --mode run --base-url ${BASE_URL} --fork-chain-version ${FORK_CHAIN_VERSION} --max-seconds ${PROBE_MAX_SECONDS} --output-json ${PROBE_REPORT}"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
sync_reason_code="not_run"
probe_reason_code="not_run"

record_check "fork_metadata_sync" "$sync_command" "planned" "not_run"
record_check "api_probe" "$probe_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "fork_metadata_sync" "$sync_command" "fail" "local_opt_in_missing"
    record_check "api_probe" "$probe_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
    sync_reason_code="local_opt_in_missing"
    probe_reason_code="local_opt_in_missing"
  else
    if bash "$SYNC_RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "$EXPECTED_REMOTE_URL" \
      --expected-ref "$EXPECTED_REF" \
      --output-json "$SYNC_REPORT" >/dev/null; then
      record_check "fork_metadata_sync" "$sync_command" "pass" "fork_metadata_verified"
      sync_reason_code="fork_metadata_verified"
    else
      sync_reason_code="$(read_report_reason_code "$SYNC_REPORT")"
      record_check "fork_metadata_sync" "$sync_command" "fail" "$sync_reason_code"
      record_check "api_probe" "$probe_command" "skipped" "fork_metadata_sync_failed"
      overall_status="fail"
      reason_code="fork_metadata_sync_failed"
      probe_reason_code="fork_metadata_sync_failed"
    fi

    if [ "$overall_status" = "ok" ]; then
      if bash "$PROBE_RUNNER" \
        --mode run \
        --base-url "$BASE_URL" \
        --fork-chain-version "$FORK_CHAIN_VERSION" \
        --max-seconds "$PROBE_MAX_SECONDS" \
        --output-json "$PROBE_REPORT" >/dev/null; then
        record_check "api_probe" "$probe_command" "pass" "probe_checks_passed"
        probe_reason_code="probe_checks_passed"
        reason_code="bootstrap_readiness_passed"
      else
        probe_reason_code="$(read_report_reason_code "$PROBE_REPORT")"
        record_check "api_probe" "$probe_command" "fail" "$probe_reason_code"
        overall_status="fail"
        reason_code="api_probe_failed"
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
      reason_code="bootstrap_budget_exceeded"
    fi
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_kolme_fork_bootstrap_readiness_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$BASE_URL" "$FORK_CHAIN_VERSION" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$SYNC_REPORT" "$PROBE_REPORT" "$sync_reason_code" "$probe_reason_code" "$CHECK_FILE"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
