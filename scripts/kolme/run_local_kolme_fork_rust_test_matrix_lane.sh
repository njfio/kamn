#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SYNC_RUNNER="$ROOT_DIR/scripts/kolme/run_local_fork_sync_metadata_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-rust-test-matrix-summary.json"
METADATA_REPORT="/tmp/kolme-local-fork-sync-metadata-summary.json"
COMMAND_OUTPUT_DIR="/tmp/kolme-local-fork-rust-test-logs"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
MAX_SECONDS=120
CARGO_PROFILE="strict"

declare -a MATRIX_COMMANDS=()

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
    --command-output-dir)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --command-output-dir" >&2
        exit 1
      fi
      COMMAND_OUTPUT_DIR="$2"
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
    --matrix-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --matrix-command" >&2
        exit 1
      fi
      MATRIX_COMMANDS+=("$2")
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
    --cargo-profile)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --cargo-profile" >&2
        exit 1
      fi
      CARGO_PROFILE="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_rust_test_matrix_lane.sh [options]

Options:
  --mode dry-run|run              Print planned matrix output or execute bounded test matrix.
  --output-json <path>            Summary report output path.
  --metadata-report <path>        Metadata lane report output path.
  --command-output-dir <path>     Directory for per-command stdout/stderr files.
  --checkout-path <path>          Local kolme_fork checkout path.
  --expected-remote-url <url>     Expected origin URL for checkout validation.
  --expected-ref <ref>            Expected symbolic HEAD ref.
  --matrix-command <command>      Repeatable bounded command to run from checkout path.
  --max-seconds <n>               Max runtime per matrix command.
  --cargo-profile <strict|portable>
                                  Cargo command execution profile for matrix commands.
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

if [ "$CARGO_PROFILE" != "strict" ] && [ "$CARGO_PROFILE" != "portable" ]; then
  echo "cargo-profile must be one of: strict, portable" >&2
  exit 1
fi

if [ -z "$CHECKOUT_PATH" ]; then
  echo "checkout path must not be empty" >&2
  exit 1
fi

if [ -z "$EXPECTED_REMOTE_URL" ]; then
  echo "expected remote URL must not be empty" >&2
  exit 1
fi

if [ -z "$EXPECTED_REF" ]; then
  echo "expected ref must not be empty" >&2
  exit 1
fi

if [ "${#MATRIX_COMMANDS[@]}" -eq 0 ]; then
  MATRIX_COMMANDS=(
    "cargo test -p merkle-map --test version -- --exact load_from_zero_example"
    "cargo test -p merkle-map --lib -- --exact insert_get"
  )
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
  local output_file="$4"
  printf '%s\t%s\t%s\t%s\n' "$check_id" "$command" "$status" "$output_file" >>"$CHECK_FILE"
}

record_check \
  "fork_sync_metadata" \
  "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" \
  "planned" \
  "$METADATA_REPORT"

for i in "${!MATRIX_COMMANDS[@]}"; do
  index="$(( i + 1 ))"
  record_check "matrix_command_$index" "${MATRIX_COMMANDS[$i]}" "planned" "$COMMAND_OUTPUT_DIR/command-$index.log"
done

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  mkdir -p "$COMMAND_OUTPUT_DIR"
  start_epoch="$(date +%s)"

  if [ ! -x "$SYNC_RUNNER" ]; then
    record_check \
      "fork_sync_metadata" \
      "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" \
      "fail" \
      "$METADATA_REPORT"
    for i in "${!MATRIX_COMMANDS[@]}"; do
      index="$(( i + 1 ))"
      record_check "matrix_command_$index" "${MATRIX_COMMANDS[$i]}" "skipped" "$COMMAND_OUTPUT_DIR/command-$index.log"
    done
    overall_status="fail"
    reason_code="fork_metadata_runner_missing"
  else
    if bash "$SYNC_RUNNER" \
      --mode run \
      --checkout-path "$CHECKOUT_PATH" \
      --expected-remote-url "$EXPECTED_REMOTE_URL" \
      --expected-ref "$EXPECTED_REF" \
      --output-json "$METADATA_REPORT" >/dev/null; then
      record_check \
        "fork_sync_metadata" \
        "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" \
        "pass" \
        "$METADATA_REPORT"

      command_failure="false"
      timeout_failure="false"
      stop_execution="false"

      for i in "${!MATRIX_COMMANDS[@]}"; do
        index="$(( i + 1 ))"
        command="${MATRIX_COMMANDS[$i]}"
        output_file="$COMMAND_OUTPUT_DIR/command-$index.log"
        effective_command="$command"

        if [ "$stop_execution" = "true" ]; then
          record_check "matrix_command_$index" "$command" "skipped" "$output_file"
          continue
        fi

        if [ "$CARGO_PROFILE" = "portable" ] && [[ "$command" == cargo\ * ]]; then
          effective_command="RUSTFLAGS='' ${command}"
        fi

        mkdir -p "$(dirname "$output_file")"
        command_exit_code=0
        set +e
        timeout "$MAX_SECONDS" bash -lc "cd \"$CHECKOUT_PATH\" && $effective_command" >"$output_file" 2>&1
        command_exit_code=$?
        set -e

        if [ "$command_exit_code" -eq 0 ]; then
          record_check "matrix_command_$index" "$command" "pass" "$output_file"
        elif [ "$command_exit_code" -eq 124 ]; then
          record_check "matrix_command_$index" "$command" "fail" "$output_file"
          timeout_failure="true"
          command_failure="true"
          stop_execution="true"
        else
          record_check "matrix_command_$index" "$command" "fail" "$output_file"
          command_failure="true"
          stop_execution="true"
        fi
      done

      if [ "$command_failure" = "true" ]; then
        overall_status="fail"
        if [ "$timeout_failure" = "true" ]; then
          reason_code="fork_rust_test_command_timeout"
          budget_status="exceeded_budget"
        else
          reason_code="fork_rust_test_command_failed"
          budget_status="within_budget"
        fi
      else
        reason_code="fork_rust_test_matrix_passed"
        budget_status="within_budget"
      fi
    else
      record_check \
        "fork_sync_metadata" \
        "bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path $CHECKOUT_PATH --expected-remote-url $EXPECTED_REMOTE_URL --expected-ref $EXPECTED_REF --output-json $METADATA_REPORT" \
        "fail" \
        "$METADATA_REPORT"
      for i in "${!MATRIX_COMMANDS[@]}"; do
        index="$(( i + 1 ))"
        record_check "matrix_command_$index" "${MATRIX_COMMANDS[$i]}" "skipped" "$COMMAND_OUTPUT_DIR/command-$index.log"
      done
      overall_status="fail"
      reason_code="fork_metadata_sync_failed"
      budget_status="within_budget"
    fi
  fi

  end_epoch="$(date +%s)"
  elapsed_seconds="$(( end_epoch - start_epoch ))"
fi

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$local_only_enforced" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$METADATA_REPORT" "$COMMAND_OUTPUT_DIR" "$CHECK_FILE" "$CARGO_PROFILE" "${MATRIX_COMMANDS[@]}" <<'PY'
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
max_seconds_per_command = int(sys.argv[7])
budget_status = sys.argv[8]
checkout_path = sys.argv[9]
expected_remote_url = sys.argv[10]
expected_ref = sys.argv[11]
metadata_report = sys.argv[12]
command_output_dir = sys.argv[13]
checks_path = pathlib.Path(sys.argv[14])
cargo_profile = sys.argv[15]
commands = sys.argv[16:]

checkpoints = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, output_file = parts
    checkpoints.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "output_file": output_file,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds_per_command": max_seconds_per_command,
    "command_count": len(commands),
    "cargo_profile": cargo_profile,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "metadata_report": metadata_report,
    "command_output_dir": command_output_dir,
    "commands": commands,
    "checkpoints": checkpoints,
    "artifact_paths": [
        metadata_report,
        command_output_dir,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "matrix_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=$local_only_enforced"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
