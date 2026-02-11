#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SYNC_RUNNER="$ROOT_DIR/scripts/kolme/run_local_fork_sync_metadata_lane.sh"
LOCAL_HEAVY_GUARD="$ROOT_DIR/scripts/framework/assert_local_heavy_opt_in.sh"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-checkout-bootstrap-summary.json"
CHECKOUT_PATH="/tmp/kolme_fork"
FORK_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
SYNC_METADATA_REPORT="/tmp/kolme-local-fork-sync-metadata-summary.json"
MAX_SECONDS=90
ALLOW_NON_DEFAULT_DIAGNOSTIC_COMMANDS="false"
GIT_VERSION_COMMAND="git --version"
CARGO_VERSION_COMMAND="cargo --version"
RUSTC_VERSION_COMMAND="rustc --version"

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
    --fork-remote-url)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --fork-remote-url" >&2
        exit 1
      fi
      FORK_REMOTE_URL="$2"
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
    --sync-metadata-report)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --sync-metadata-report" >&2
        exit 1
      fi
      SYNC_METADATA_REPORT="$2"
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
    --git-version-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --git-version-command" >&2
        exit 1
      fi
      GIT_VERSION_COMMAND="$2"
      shift 2
      ;;
    --cargo-version-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --cargo-version-command" >&2
        exit 1
      fi
      CARGO_VERSION_COMMAND="$2"
      shift 2
      ;;
    --rustc-version-command)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --rustc-version-command" >&2
        exit 1
      fi
      RUSTC_VERSION_COMMAND="$2"
      shift 2
      ;;
    --allow-non-default-diagnostic-commands)
      ALLOW_NON_DEFAULT_DIAGNOSTIC_COMMANDS="true"
      shift 1
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_kolme_fork_checkout_bootstrap_lane.sh [options]

Options:
  --mode dry-run|run                              Emit planned checks or execute checkout bootstrap.
  --output-json <path>                            Deterministic summary output path.
  --checkout-path <path>                          Local checkout path for `njfio/kolme_fork`.
  --fork-remote-url <url-or-path>                 Clone/fetch source used to bootstrap checkout.
  --expected-remote-url <url-or-path>             Expected origin URL used by sync metadata verification.
  --expected-ref <ref>                            Expected symbolic HEAD ref (for example refs/heads/main).
  --sync-metadata-report <path>                   Output path for nested sync metadata summary.
  --max-seconds <n>                               Max runtime budget for run mode.
  --git-version-command <command>                 Optional override for git diagnostics command.
  --cargo-version-command <command>               Optional override for cargo diagnostics command.
  --rustc-version-command <command>               Optional override for rustc diagnostics command.
  --allow-non-default-diagnostic-commands         Allow diagnostic command overrides for local harness tests.
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

if [ -z "$CHECKOUT_PATH" ] || [ -z "$FORK_REMOTE_URL" ] || [ -z "$EXPECTED_REMOTE_URL" ] || [ -z "$EXPECTED_REF" ]; then
  echo "checkout-path, fork-remote-url, expected-remote-url, and expected-ref must not be empty" >&2
  exit 1
fi

if [ ! -x "$SYNC_RUNNER" ]; then
  echo "expected local fork sync metadata runner to be executable" >&2
  exit 1
fi

if [ ! -x "$LOCAL_HEAVY_GUARD" ]; then
  echo "expected shared local-heavy opt-in guard helper to be executable" >&2
  exit 1
fi

if [[ "$EXPECTED_REF" != refs/heads/* ]]; then
  echo "expected-ref must use refs/heads/* format" >&2
  exit 1
fi

EXPECTED_BRANCH="${EXPECTED_REF#refs/heads/}"
if [ -z "$EXPECTED_BRANCH" ]; then
  echo "expected-ref must include branch name" >&2
  exit 1
fi

DEFAULT_GIT_VERSION_COMMAND="git --version"
DEFAULT_CARGO_VERSION_COMMAND="cargo --version"
DEFAULT_RUSTC_VERSION_COMMAND="rustc --version"

if [ "$ALLOW_NON_DEFAULT_DIAGNOSTIC_COMMANDS" != "true" ] && {
  [ "$GIT_VERSION_COMMAND" != "$DEFAULT_GIT_VERSION_COMMAND" ] || \
  [ "$CARGO_VERSION_COMMAND" != "$DEFAULT_CARGO_VERSION_COMMAND" ] || \
  [ "$RUSTC_VERSION_COMMAND" != "$DEFAULT_RUSTC_VERSION_COMMAND" ];
}; then
  echo "diagnostic command overrides require --allow-non-default-diagnostic-commands" >&2
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

read_reason_code() {
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

reason_code = payload.get("reason_code")
if isinstance(reason_code, str) and reason_code.strip():
    print(reason_code)
else:
    print("reason_code_missing")
PY
}

run_command() {
  local command="$1"
  local output_file="$2"
  local exit_code=0

  set +e
  timeout "$MAX_SECONDS" bash -lc "$command" >"$output_file" 2>&1
  exit_code=$?
  set -e
  return "$exit_code"
}

sanitize_single_line() {
  local value="$1"
  value="${value//$'\r'/}"
  value="${value//$'\n'/ }"
  printf '%s' "$value"
}

checkout_prepare_command="if checkout exists: git -C \"$CHECKOUT_PATH\" fetch origin \"$EXPECTED_BRANCH\" && git -C \"$CHECKOUT_PATH\" checkout -B \"$EXPECTED_BRANCH\" \"origin/$EXPECTED_BRANCH\"; else: git clone --depth 1 --branch \"$EXPECTED_BRANCH\" \"$FORK_REMOTE_URL\" \"$CHECKOUT_PATH\""
sync_metadata_command="bash scripts/kolme/run_local_fork_sync_metadata_lane.sh --mode run --checkout-path \"$CHECKOUT_PATH\" --expected-remote-url \"$EXPECTED_REMOTE_URL\" --expected-ref \"$EXPECTED_REF\" --output-json \"$SYNC_METADATA_REPORT\""
git_version_command="$GIT_VERSION_COMMAND"
cargo_version_command="$CARGO_VERSION_COMMAND"
rustc_version_command="$RUSTC_VERSION_COMMAND"

overall_status="ok"
reason_code="dry_run_no_commands_executed"
budget_status="not_run"
elapsed_seconds=0
bootstrap_action="planned"
git_version="not_run"
cargo_version="not_run"
rustc_version="not_run"

record_check "checkout_prepare" "$checkout_prepare_command" "planned" "not_run"
record_check "sync_metadata" "$sync_metadata_command" "planned" "not_run"
record_check "diagnostics_git_version" "$git_version_command" "planned" "not_run"
record_check "diagnostics_cargo_version" "$cargo_version_command" "planned" "not_run"
record_check "diagnostics_rustc_version" "$rustc_version_command" "planned" "not_run"

if [ "$MODE" = "run" ]; then
  : >"$CHECK_FILE"
  start_epoch="$(date +%s)"

  if ! "$LOCAL_HEAVY_GUARD"; then
    record_check "checkout_prepare" "$checkout_prepare_command" "fail" "local_opt_in_missing"
    record_check "sync_metadata" "$sync_metadata_command" "skipped" "local_opt_in_missing"
    record_check "diagnostics_git_version" "$git_version_command" "skipped" "local_opt_in_missing"
    record_check "diagnostics_cargo_version" "$cargo_version_command" "skipped" "local_opt_in_missing"
    record_check "diagnostics_rustc_version" "$rustc_version_command" "skipped" "local_opt_in_missing"
    overall_status="fail"
    reason_code="local_opt_in_missing"
  else
    checkout_prepare_output="$(mktemp)"
    if [ -d "$CHECKOUT_PATH/.git" ]; then
      checkout_prepare_exec="git -C $(printf '%q' "$CHECKOUT_PATH") fetch origin $(printf '%q' "$EXPECTED_BRANCH") && git -C $(printf '%q' "$CHECKOUT_PATH") checkout -B $(printf '%q' "$EXPECTED_BRANCH") origin/$(printf '%q' "$EXPECTED_BRANCH")"
      if run_command "$checkout_prepare_exec" "$checkout_prepare_output"; then
        record_check "checkout_prepare" "$checkout_prepare_command" "pass" "checkout_updated"
        bootstrap_action="updated"
      else
        checkout_prepare_exit_code=$?
        if [ "$checkout_prepare_exit_code" -eq 124 ]; then
          record_check "checkout_prepare" "$checkout_prepare_command" "fail" "checkout_prepare_timeout"
        else
          record_check "checkout_prepare" "$checkout_prepare_command" "fail" "checkout_prepare_failed"
        fi
        record_check "sync_metadata" "$sync_metadata_command" "skipped" "checkout_prepare_failed"
        record_check "diagnostics_git_version" "$git_version_command" "skipped" "checkout_prepare_failed"
        record_check "diagnostics_cargo_version" "$cargo_version_command" "skipped" "checkout_prepare_failed"
        record_check "diagnostics_rustc_version" "$rustc_version_command" "skipped" "checkout_prepare_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_checkout_prepare"
      fi
    elif [ -e "$CHECKOUT_PATH" ]; then
      record_check "checkout_prepare" "$checkout_prepare_command" "fail" "checkout_path_not_git_repo"
      record_check "sync_metadata" "$sync_metadata_command" "skipped" "checkout_prepare_failed"
      record_check "diagnostics_git_version" "$git_version_command" "skipped" "checkout_prepare_failed"
      record_check "diagnostics_cargo_version" "$cargo_version_command" "skipped" "checkout_prepare_failed"
      record_check "diagnostics_rustc_version" "$rustc_version_command" "skipped" "checkout_prepare_failed"
      overall_status="fail"
      reason_code="checkpoint_failed_checkout_prepare"
    else
      mkdir -p "$(dirname "$CHECKOUT_PATH")"
      checkout_prepare_exec="git clone --depth 1 --branch $(printf '%q' "$EXPECTED_BRANCH") $(printf '%q' "$FORK_REMOTE_URL") $(printf '%q' "$CHECKOUT_PATH")"
      if run_command "$checkout_prepare_exec" "$checkout_prepare_output"; then
        record_check "checkout_prepare" "$checkout_prepare_command" "pass" "checkout_cloned"
        bootstrap_action="cloned"
      else
        checkout_prepare_exit_code=$?
        if [ "$checkout_prepare_exit_code" -eq 124 ]; then
          record_check "checkout_prepare" "$checkout_prepare_command" "fail" "checkout_prepare_timeout"
        else
          record_check "checkout_prepare" "$checkout_prepare_command" "fail" "checkout_prepare_failed"
        fi
        record_check "sync_metadata" "$sync_metadata_command" "skipped" "checkout_prepare_failed"
        record_check "diagnostics_git_version" "$git_version_command" "skipped" "checkout_prepare_failed"
        record_check "diagnostics_cargo_version" "$cargo_version_command" "skipped" "checkout_prepare_failed"
        record_check "diagnostics_rustc_version" "$rustc_version_command" "skipped" "checkout_prepare_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_checkout_prepare"
      fi
    fi
    rm -f "$checkout_prepare_output"

    if [ "$overall_status" = "ok" ]; then
      sync_output="$(mktemp)"
      sync_exec="bash $(printf '%q' "$SYNC_RUNNER") --mode run --checkout-path $(printf '%q' "$CHECKOUT_PATH") --expected-remote-url $(printf '%q' "$EXPECTED_REMOTE_URL") --expected-ref $(printf '%q' "$EXPECTED_REF") --output-json $(printf '%q' "$SYNC_METADATA_REPORT")"
      if run_command "$sync_exec" "$sync_output"; then
        record_check "sync_metadata" "$sync_metadata_command" "pass" "sync_metadata_passed"
      else
        sync_exit_code=$?
        sync_reason_code="$(read_reason_code "$SYNC_METADATA_REPORT")"
        if [ "$sync_exit_code" -eq 124 ]; then
          record_check "sync_metadata" "$sync_metadata_command" "fail" "sync_metadata_timeout"
        else
          record_check "sync_metadata" "$sync_metadata_command" "fail" "$sync_reason_code"
        fi
        record_check "diagnostics_git_version" "$git_version_command" "skipped" "sync_metadata_failed"
        record_check "diagnostics_cargo_version" "$cargo_version_command" "skipped" "sync_metadata_failed"
        record_check "diagnostics_rustc_version" "$rustc_version_command" "skipped" "sync_metadata_failed"
        overall_status="fail"
        reason_code="checkpoint_failed_sync_metadata"
      fi
      rm -f "$sync_output"
    fi

    run_diagnostic_check() {
      local check_id="$1"
      local command="$2"
      local checkpoint_reason="$3"
      local fail_reason="$4"
      local timeout_reason="$5"
      local output_ref_name="$6"
      local output_file
      output_file="$(mktemp)"

      if run_command "$command" "$output_file"; then
        local command_output
        command_output="$(head -n 1 "$output_file" || true)"
        command_output="$(sanitize_single_line "$command_output")"
        if [ -z "$command_output" ]; then
          command_output="command_output_empty"
        fi
        record_check "$check_id" "$command" "pass" "${check_id}_passed"
        printf -v "$output_ref_name" '%s' "$command_output"
      else
        local command_exit_code=$?
        if [ "$command_exit_code" -eq 124 ]; then
          record_check "$check_id" "$command" "fail" "$timeout_reason"
        else
          record_check "$check_id" "$command" "fail" "$fail_reason"
        fi
        overall_status="fail"
        reason_code="$checkpoint_reason"
      fi

      rm -f "$output_file"
    }

    if [ "$overall_status" = "ok" ]; then
      run_diagnostic_check \
        "diagnostics_git_version" \
        "$GIT_VERSION_COMMAND" \
        "checkpoint_failed_git_version" \
        "git_version_failed" \
        "git_version_timeout" \
        "git_version"
    fi

    if [ "$overall_status" = "ok" ]; then
      run_diagnostic_check \
        "diagnostics_cargo_version" \
        "$CARGO_VERSION_COMMAND" \
        "checkpoint_failed_cargo_version" \
        "cargo_version_failed" \
        "cargo_version_timeout" \
        "cargo_version"
    fi

    if [ "$overall_status" = "ok" ]; then
      run_diagnostic_check \
        "diagnostics_rustc_version" \
        "$RUSTC_VERSION_COMMAND" \
        "checkpoint_failed_rustc_version" \
        "rustc_version_failed" \
        "rustc_version_timeout" \
        "rustc_version"
    fi

    if [ "$overall_status" = "ok" ]; then
      if [ "$bootstrap_action" = "planned" ]; then
        bootstrap_action="validated"
      fi
      reason_code="fork_checkout_bootstrap_passed"
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

python3 - "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$elapsed_seconds" "$MAX_SECONDS" "$budget_status" "$CHECKOUT_PATH" "$FORK_REMOTE_URL" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$bootstrap_action" "$SYNC_METADATA_REPORT" "$git_version" "$cargo_version" "$rustc_version" "$CHECK_FILE" <<'PY'
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
checkout_path = sys.argv[8]
fork_remote_url = sys.argv[9]
expected_remote_url = sys.argv[10]
expected_ref = sys.argv[11]
bootstrap_action = sys.argv[12]
sync_metadata_report = sys.argv[13]
git_version = sys.argv[14]
cargo_version = sys.argv[15]
rustc_version = sys.argv[16]
checks_path = pathlib.Path(sys.argv[17])

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
    "schema_version": "kamn.kolme.local-fork-checkout-bootstrap-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "fork_remote_url": fork_remote_url,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "bootstrap_action": bootstrap_action,
    "sync_metadata_report": sync_metadata_report,
    "diagnostics": {
        "git_version": git_version,
        "cargo_version": cargo_version,
        "rustc_version": rustc_version,
    },
    "checks": checks,
    "artifact_paths": [
        sync_metadata_report,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

echo "status=$overall_status"
echo "bootstrap_mode=$MODE"
echo "reason_code=$reason_code"
echo "budget_status=$budget_status"
echo "local_only_enforced=true"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
