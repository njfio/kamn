#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SYNC_RUNNER="$ROOT_DIR/scripts/kolme/run_local_fork_sync_metadata_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-smoke-evidence-summary.json"
METADATA_REPORT="/tmp/kolme-local-fork-sync-metadata-summary.json"
SMOKE_OUTPUT_FILE="/tmp/kolme-local-fork-smoke-output.txt"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
SMOKE_COMMAND="cargo test -p merkle-map --test version -- --exact load_from_zero_example"
MAX_SECONDS=120

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
    --metadata-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --metadata-report" >&2
        exit 1
      fi
      METADATA_REPORT="$2"
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
    --smoke-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --smoke-command" >&2
        exit 1
      fi
      SMOKE_COMMAND="$2"
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
Usage: run_local_fork_smoke_evidence_lane.sh [options]

Options:
  --mode dry-run|run              Run planned output only, or execute smoke lane.
  --output-json <path>            Summary report output path.
  --metadata-report <path>        Metadata lane report output path.
  --smoke-output-file <path>      Captured stdout/stderr for smoke command.
  --checkout-path <path>          Local kolme_fork checkout path.
  --expected-remote-url <url>     Expected origin URL for checkout validation.
  --expected-ref <ref>            Expected symbolic HEAD ref.
  --smoke-command <command>       Bounded smoke command to execute in checkout path.
  --max-seconds <n>               Max runtime for smoke command (run mode only).
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

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ "$MODE" = "run" ]; then
  "$LOCAL_HEAVY_GUARD"
fi

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
local_only_enforced="true"

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  printf '%s\t%s\t%s\n' "$check_id" "$command" "$status" >>"$CHECK_FILE"
}

record_check "fork_sync_metadata" "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" "planned"
record_check "fork_smoke_command" "$SMOKE_COMMAND" "planned"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if [ ! -x "$SYNC_RUNNER" ]; then
    record_check "fork_sync_metadata" "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" "fail"
    record_check "fork_smoke_command" "$SMOKE_COMMAND" "skipped"
    overall_status="fail"
    reason_code="fork_metadata_runner_missing"
  else
    if bash "$SYNC_RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "$EXPECTED_REMOTE_URL" \
      --expected-ref "$EXPECTED_REF" \
      --output-json "$METADATA_REPORT" >/dev/null; then
      record_check "fork_sync_metadata" "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" "pass"

      smoke_exit_code=0
      mkdir -p "$(dirname "$SMOKE_OUTPUT_FILE")"
      set +e
      timeout "${MAX_SECONDS}" bash -lc "cd \"$CHECKOUT_PATH\" && $SMOKE_COMMAND" >"$SMOKE_OUTPUT_FILE" 2>&1
      smoke_exit_code=$?
      set -e

      if [ "$smoke_exit_code" -eq 0 ]; then
        record_check "fork_smoke_command" "$SMOKE_COMMAND" "pass"
        reason_code="fork_smoke_command_passed"
      elif [ "$smoke_exit_code" -eq 124 ]; then
        record_check "fork_smoke_command" "$SMOKE_COMMAND" "fail"
        overall_status="fail"
        reason_code="fork_smoke_command_timeout"
      else
        record_check "fork_smoke_command" "$SMOKE_COMMAND" "fail"
        overall_status="fail"
        reason_code="fork_smoke_command_failed"
      fi
    else
      record_check "fork_sync_metadata" "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" "fail"
      record_check "fork_smoke_command" "$SMOKE_COMMAND" "skipped"
      overall_status="fail"
      reason_code="fork_metadata_sync_failed"
    fi
  fi

  end_epoch="$(date +%s)"
  elapsed_seconds="$(( end_epoch - start_epoch ))"
  if [ "$elapsed_seconds" -le "$MAX_SECONDS" ] && [ "$reason_code" != "fork_smoke_command_timeout" ]; then
    budget_status="within_budget"
  else
    budget_status="exceeded_budget"
    if [ "$overall_status" = "ok" ]; then
      overall_status="fail"
      reason_code="fork_smoke_budget_exceeded"
    fi
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_fork_smoke_evidence_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$local_only_enforced" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$SMOKE_COMMAND" "$METADATA_REPORT" "$SMOKE_OUTPUT_FILE" "$CHECK_FILE"

echo "status=$overall_status"
echo "smoke_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=$local_only_enforced"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
