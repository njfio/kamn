#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SUMMARY_HELPER="$ROOT_DIR/scripts/framework/generate_local_lane_summary.py"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-portability-preflight-summary.json"
CHECKOUT_PATH="/tmp/kolme_fork"
MAX_SECONDS=300
KOLME_COMPILE_COMMAND="RUSTFLAGS='' cargo test -p kolme --locked --no-run"
INTEGRATION_COMPILE_COMMAND="RUSTFLAGS='' cargo test -p integration-tests --test six-sigma --locked --no-run"
LINKER_PROBE_COMMAND="command -v mold >/dev/null 2>&1 || command -v ld.mold >/dev/null 2>&1"
LIBUDEV_PROBE_COMMAND="pkg-config --libs --cflags libudev"

KOLME_BUILD_LOG="/tmp/kolme-local-fork-portability-preflight-kolme-build.log"
INTEGRATION_BUILD_LOG="/tmp/kolme-local-fork-portability-preflight-integration-build.log"
LINKER_PROBE_LOG="/tmp/kolme-local-fork-portability-preflight-linker.log"
LIBUDEV_PROBE_LOG="/tmp/kolme-local-fork-portability-preflight-libudev.log"

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
    --checkout-path)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --checkout-path" >&2
        exit 1
      fi
      CHECKOUT_PATH="$2"
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
    --kolme-compile-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --kolme-compile-command" >&2
        exit 1
      fi
      KOLME_COMPILE_COMMAND="$2"
      shift 2
      ;;
    --integration-compile-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-compile-command" >&2
        exit 1
      fi
      INTEGRATION_COMPILE_COMMAND="$2"
      shift 2
      ;;
    --linker-probe-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --linker-probe-command" >&2
        exit 1
      fi
      LINKER_PROBE_COMMAND="$2"
      shift 2
      ;;
    --libudev-probe-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --libudev-probe-command" >&2
        exit 1
      fi
      LIBUDEV_PROBE_COMMAND="$2"
      shift 2
      ;;
    --kolme-build-log)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --kolme-build-log" >&2
        exit 1
      fi
      KOLME_BUILD_LOG="$2"
      shift 2
      ;;
    --integration-build-log)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --integration-build-log" >&2
        exit 1
      fi
      INTEGRATION_BUILD_LOG="$2"
      shift 2
      ;;
    --linker-probe-log)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --linker-probe-log" >&2
        exit 1
      fi
      LINKER_PROBE_LOG="$2"
      shift 2
      ;;
    --libudev-probe-log)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --libudev-probe-log" >&2
        exit 1
      fi
      LIBUDEV_PROBE_LOG="$2"
      shift 2
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_portability_preflight_lane.sh [options]

Options:
  --mode dry-run|run                        Emit planned checkpoints or execute local portability probes.
  --output-json <path>                      Deterministic summary output path.
  --checkout-path <path>                    Local kolm_fork checkout path.
  --max-seconds <n>                         Total runtime budget for run mode.
  --kolme-compile-command <command>         Compile command for `kolme` crate probe.
  --integration-compile-command <command>   Compile command for integration-test probe.
  --linker-probe-command <command>          Command used to verify `mold` linker availability.
  --libudev-probe-command <command>         Command used to verify `libudev` availability.
  --kolme-build-log <path>                  Output log for `kolme` compile probe.
  --integration-build-log <path>            Output log for integration compile probe.
  --linker-probe-log <path>                 Output log for linker probe.
  --libudev-probe-log <path>                Output log for libudev probe.
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
  echo "max seconds must be a positive integer" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [ ! -x "$SUMMARY_HELPER" ]; then
  echo "expected shared local-lane summary helper to be executable" >&2
  exit 1
fi

CHECKPOINT_FILE="$(mktemp)"
ARTIFACT_FILE="$(mktemp)"
trap 'rm -f "$CHECKPOINT_FILE" "$ARTIFACT_FILE"' EXIT

record_checkpoint() {
  local checkpoint_id="$1"
  local checkpoint_command="$2"
  local checkpoint_status="$3"
  printf '%s\t%s\t%s\n' "$checkpoint_id" "$checkpoint_command" "$checkpoint_status" >>"$CHECKPOINT_FILE"
}

local_opt_in_command="bash scripts/framework/assert_local_heavy_opt_in.sh"
mold_linker_probe_command="bash -lc '$LINKER_PROBE_COMMAND'"
kolme_compile_probe_command="bash -lc 'cd $CHECKOUT_PATH && $KOLME_COMPILE_COMMAND'"
libudev_probe_command="bash -lc '$LIBUDEV_PROBE_COMMAND'"
integration_compile_probe_command="bash -lc 'cd $CHECKOUT_PATH && $INTEGRATION_COMPILE_COMMAND'"

record_checkpoint "local_opt_in_guard" "$local_opt_in_command" "planned"
record_checkpoint "mold_linker_probe" "$mold_linker_probe_command" "planned"
record_checkpoint "kolme_compile_probe" "$kolme_compile_probe_command" "planned"
record_checkpoint "libudev_probe" "$libudev_probe_command" "planned"
record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "planned"

printf '%s\n' "$LINKER_PROBE_LOG" >>"$ARTIFACT_FILE"
printf '%s\n' "$KOLME_BUILD_LOG" >>"$ARTIFACT_FILE"
printf '%s\n' "$LIBUDEV_PROBE_LOG" >>"$ARTIFACT_FILE"
printf '%s\n' "$INTEGRATION_BUILD_LOG" >>"$ARTIFACT_FILE"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
start_epoch="$(date +%s)"
budget_status="pass"

if [ "$MODE" = "run" ]; then
  : >"$CHECKPOINT_FILE"

  set +e
  local_opt_in_output="$("$LOCAL_HEAVY_GUARD" 2>&1)"
  local_opt_in_exit_code=$?
  set -e
  printf '%s\n' "$local_opt_in_output" >"$LINKER_PROBE_LOG"
  if [ "$local_opt_in_exit_code" -ne 0 ] && [ -n "$local_opt_in_output" ]; then
    printf '%s\n' "$local_opt_in_output" >&2
  fi

  if [ "$local_opt_in_exit_code" -eq 0 ]; then
    record_checkpoint "local_opt_in_guard" "$local_opt_in_command" "pass"
  else
    record_checkpoint "local_opt_in_guard" "$local_opt_in_command" "fail"
    record_checkpoint "mold_linker_probe" "$mold_linker_probe_command" "skipped"
    record_checkpoint "kolme_compile_probe" "$kolme_compile_probe_command" "skipped"
    record_checkpoint "libudev_probe" "$libudev_probe_command" "skipped"
    record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "skipped"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  fi

  if [ "$overall_status" = "ok" ] && [ ! -d "$CHECKOUT_PATH" ]; then
    record_checkpoint "mold_linker_probe" "$mold_linker_probe_command" "fail"
    record_checkpoint "kolme_compile_probe" "$kolme_compile_probe_command" "skipped"
    record_checkpoint "libudev_probe" "$libudev_probe_command" "skipped"
    record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "skipped"
    overall_status="fail"
    reason_code="checkout_path_missing"
  fi

  if [ "$overall_status" = "ok" ]; then
    requires_mold="false"
    if [ -f "$CHECKOUT_PATH/.cargo/config.toml" ] && grep -q "-fuse-ld=mold" "$CHECKOUT_PATH/.cargo/config.toml"; then
      requires_mold="true"
    fi

    if [ "$requires_mold" = "true" ]; then
      set +e
      timeout "$MAX_SECONDS" bash -lc "$LINKER_PROBE_COMMAND" >"$LINKER_PROBE_LOG" 2>&1
      linker_probe_exit_code=$?
      set -e

      if [ "$linker_probe_exit_code" -eq 0 ]; then
        record_checkpoint "mold_linker_probe" "$mold_linker_probe_command" "pass"
      else
        record_checkpoint "mold_linker_probe" "$mold_linker_probe_command" "fail"
        record_checkpoint "kolme_compile_probe" "$kolme_compile_probe_command" "skipped"
        record_checkpoint "libudev_probe" "$libudev_probe_command" "skipped"
        record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "skipped"
        overall_status="fail"
        reason_code="checkpoint_failed_mold_linker_probe"
      fi
    else
      record_checkpoint "mold_linker_probe" "mold_not_required" "pass"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    set +e
    timeout "$MAX_SECONDS" bash -lc "cd $(printf '%q' "$CHECKOUT_PATH") && $KOLME_COMPILE_COMMAND" >"$KOLME_BUILD_LOG" 2>&1
    kolme_compile_exit_code=$?
    set -e

    if [ "$kolme_compile_exit_code" -eq 0 ]; then
      record_checkpoint "kolme_compile_probe" "$kolme_compile_probe_command" "pass"
    else
      record_checkpoint "kolme_compile_probe" "$kolme_compile_probe_command" "fail"
      record_checkpoint "libudev_probe" "$libudev_probe_command" "skipped"
      record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "skipped"
      overall_status="fail"
      reason_code="checkpoint_failed_kolme_compile_probe"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    set +e
    timeout "$MAX_SECONDS" bash -lc "$LIBUDEV_PROBE_COMMAND" >"$LIBUDEV_PROBE_LOG" 2>&1
    libudev_probe_exit_code=$?
    set -e

    if [ "$libudev_probe_exit_code" -eq 0 ]; then
      record_checkpoint "libudev_probe" "$libudev_probe_command" "pass"
    else
      record_checkpoint "libudev_probe" "$libudev_probe_command" "fail"
      record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "skipped"
      overall_status="fail"
      reason_code="checkpoint_failed_libudev_probe"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    set +e
    timeout "$MAX_SECONDS" bash -lc "cd $(printf '%q' "$CHECKOUT_PATH") && $INTEGRATION_COMPILE_COMMAND" >"$INTEGRATION_BUILD_LOG" 2>&1
    integration_compile_exit_code=$?
    set -e

    if [ "$integration_compile_exit_code" -eq 0 ]; then
      record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "pass"
    else
      record_checkpoint "integration_compile_probe" "$integration_compile_probe_command" "fail"
      overall_status="fail"
      reason_code="checkpoint_failed_integration_compile_probe"
    fi
  fi

  if [ "$overall_status" = "ok" ]; then
    reason_code="portability_preflight_passed"
  fi
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$MAX_SECONDS" ]; then
  budget_status="fail"
  if [ "$overall_status" = "ok" ]; then
    overall_status="fail"
    reason_code="runtime_budget_exceeded"
  fi
fi

python3 "$SUMMARY_HELPER" \
  --schema-version "kamn.kolme.local-fork-portability-preflight-summary.v1" \
  --summary-type checkpoints \
  --mode "$MODE" \
  --status "$overall_status" \
  --reason-code "$reason_code" \
  --local-only-enforced true \
  --checkpoints-file "$CHECKPOINT_FILE" \
  --artifacts-file "$ARTIFACT_FILE" \
  --elapsed-seconds "$elapsed_seconds" \
  --max-seconds "$MAX_SECONDS" \
  --budget-status "$budget_status" \
  --output-json "$OUTPUT_JSON"

echo "status=$overall_status"
echo "lane_mode=$MODE"
echo "local_only_enforced=true"
echo "elapsed_seconds=$elapsed_seconds"
echo "max_seconds=$MAX_SECONDS"
echo "budget_status=$budget_status"
if [ -n "$reason_code" ]; then
  echo "reason_code=$reason_code"
fi
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
