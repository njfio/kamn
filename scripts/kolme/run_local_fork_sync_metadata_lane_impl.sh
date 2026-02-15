#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MODE="dry-run"
OUTPUT_JSON="/tmp/kolme-local-fork-sync-metadata-summary.json"
CHECKOUT_PATH="/tmp/kolme_fork"
EXPECTED_REMOTE_URL="https://github.com/njfio/kolme_fork.git"
EXPECTED_REF="refs/heads/main"
EXPECTED_COMMIT=""
ALLOW_DIRTY="false"

normalize_remote_url() {
  local value="$1"
  local normalized="$value"

  if [[ "$normalized" == git@github.com:* ]]; then
    normalized="https://github.com/${normalized#git@github.com:}"
  fi

  normalized="${normalized%.git}"
  printf '%s\n' "$normalized"
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
    --expected-commit)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --expected-commit" >&2
        exit 1
      fi
      EXPECTED_COMMIT="$2"
      shift 2
      ;;
    --allow-dirty)
      ALLOW_DIRTY="true"
      shift
      ;;
    --help|-h)
      cat <<'USAGE'
Usage: run_local_fork_sync_metadata_lane.sh [options]

Options:
  --mode dry-run|run              Choose dry-run plan output or active metadata validation.
  --output-json <path>            Write deterministic summary JSON to this path.
  --checkout-path <path>          Path to local kolme_fork checkout (default: /tmp/kolme_fork).
  --expected-remote-url <url>     Expected origin URL for the checkout.
  --expected-ref <ref>            Expected HEAD symbolic ref (default: refs/heads/main).
  --expected-commit <sha>         Optional pinned HEAD commit SHA (40-hex) for drift checks.
  --allow-dirty                   Allow a dirty checkout to pass metadata validation.
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

if [ -z "${CHECKOUT_PATH}" ]; then
  echo "checkout path must not be empty" >&2
  exit 1
fi

if [ -z "${EXPECTED_REMOTE_URL}" ]; then
  echo "expected remote URL must not be empty" >&2
  exit 1
fi

if [ -z "${EXPECTED_REF}" ]; then
  echo "expected ref must not be empty" >&2
  exit 1
fi
if [ -n "${EXPECTED_COMMIT}" ] && ! [[ "${EXPECTED_COMMIT}" =~ ^[0-9a-fA-F]{40}$ ]]; then
  echo "expected-commit must be a 40-character hex SHA when provided" >&2
  exit 1
fi

EXPECTED_REMOTE_URL_NORMALIZED="$(normalize_remote_url "$EXPECTED_REMOTE_URL")"

CHECK_FILE="$(mktemp)"
trap 'rm -f "$CHECK_FILE"' EXIT

overall_status="ok"
reason_code=""
metadata_verified="false"
remote_url=""
head_ref=""
head_commit=""
dirty_checkout="false"
check_failed=0

record_check() {
  local check_id="$1"
  local command="$2"
  local status="$3"
  printf '%s\t%s\t%s\n' "$check_id" "$command" "$status" >>"$CHECK_FILE"
}

fail_check() {
  local check_id="$1"
  local reason="$2"
  local command="$3"
  check_failed=1
  overall_status="fail"
  reason_code="$reason"
  record_check "$check_id" "$command" "fail"
}

if [ "$MODE" = "dry-run" ]; then
  record_check "checkout_exists" "test -d \"$CHECKOUT_PATH\"" "planned"
  record_check "git_repository" "git -C \"$CHECKOUT_PATH\" rev-parse --is-inside-work-tree" "planned"
  record_check "origin_remote_matches" "git -C \"$CHECKOUT_PATH\" remote get-url origin" "planned"
  record_check "head_ref_matches" "git -C \"$CHECKOUT_PATH\" symbolic-ref -q HEAD" "planned"
  record_check "head_commit_available" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD" "planned"
  record_check "head_commit_matches" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD == \"$EXPECTED_COMMIT\"" "planned"
  record_check "checkout_clean" "git -C \"$CHECKOUT_PATH\" status --porcelain --untracked-files=no" "planned"
  reason_code="dry_run_no_commands_executed"
else
  if [ -d "$CHECKOUT_PATH" ]; then
    record_check "checkout_exists" "test -d \"$CHECKOUT_PATH\"" "pass"
  else
    fail_check "checkout_exists" "checkout_path_missing" "test -d \"$CHECKOUT_PATH\""
  fi

  if [ "$check_failed" -eq 0 ]; then
    if git -C "$CHECKOUT_PATH" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
      record_check "git_repository" "git -C \"$CHECKOUT_PATH\" rev-parse --is-inside-work-tree" "pass"
    else
      fail_check "git_repository" "checkout_not_git_repo" "git -C \"$CHECKOUT_PATH\" rev-parse --is-inside-work-tree"
    fi
  else
    record_check "git_repository" "git -C \"$CHECKOUT_PATH\" rev-parse --is-inside-work-tree" "skipped"
  fi

  if [ "$check_failed" -eq 0 ]; then
    remote_url="$(git -C "$CHECKOUT_PATH" remote get-url origin 2>/dev/null || true)"
    remote_url_normalized="$(normalize_remote_url "$remote_url")"
    if [ -n "$remote_url" ] && [ "$remote_url_normalized" = "$EXPECTED_REMOTE_URL_NORMALIZED" ]; then
      record_check "origin_remote_matches" "git -C \"$CHECKOUT_PATH\" remote get-url origin" "pass"
    else
      fail_check "origin_remote_matches" "remote_url_mismatch" "git -C \"$CHECKOUT_PATH\" remote get-url origin"
    fi
  else
    record_check "origin_remote_matches" "git -C \"$CHECKOUT_PATH\" remote get-url origin" "skipped"
  fi

  if [ "$check_failed" -eq 0 ]; then
    head_ref="$(git -C "$CHECKOUT_PATH" symbolic-ref -q HEAD 2>/dev/null || true)"
    if [ -n "$head_ref" ] && [ "$head_ref" = "$EXPECTED_REF" ]; then
      record_check "head_ref_matches" "git -C \"$CHECKOUT_PATH\" symbolic-ref -q HEAD" "pass"
    else
      fail_check "head_ref_matches" "head_ref_mismatch" "git -C \"$CHECKOUT_PATH\" symbolic-ref -q HEAD"
    fi
  else
    record_check "head_ref_matches" "git -C \"$CHECKOUT_PATH\" symbolic-ref -q HEAD" "skipped"
  fi

  if [ "$check_failed" -eq 0 ]; then
    head_commit="$(git -C "$CHECKOUT_PATH" rev-parse HEAD 2>/dev/null || true)"
    if [ -n "$head_commit" ]; then
      record_check "head_commit_available" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD" "pass"
    else
      fail_check "head_commit_available" "head_commit_missing" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD"
    fi
  else
    record_check "head_commit_available" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD" "skipped"
  fi

  if [ "$check_failed" -eq 0 ]; then
    if [ -n "$EXPECTED_COMMIT" ]; then
      if [ "$head_commit" = "$EXPECTED_COMMIT" ]; then
        record_check "head_commit_matches" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD == \"$EXPECTED_COMMIT\"" "pass"
      else
        fail_check "head_commit_matches" "head_commit_mismatch" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD == \"$EXPECTED_COMMIT\""
      fi
    else
      record_check "head_commit_matches" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD == \"$EXPECTED_COMMIT\"" "skipped"
    fi
  else
    record_check "head_commit_matches" "git -C \"$CHECKOUT_PATH\" rev-parse HEAD == \"$EXPECTED_COMMIT\"" "skipped"
  fi

  if [ "$check_failed" -eq 0 ]; then
    dirty_output="$(git -C "$CHECKOUT_PATH" status --porcelain --untracked-files=no 2>/dev/null || true)"
    if [ -n "$dirty_output" ]; then
      dirty_checkout="true"
    fi
    if [ "$dirty_checkout" = "true" ] && [ "$ALLOW_DIRTY" != "true" ]; then
      fail_check "checkout_clean" "checkout_dirty" "git -C \"$CHECKOUT_PATH\" status --porcelain --untracked-files=no"
    else
      record_check "checkout_clean" "git -C \"$CHECKOUT_PATH\" status --porcelain --untracked-files=no" "pass"
    fi
  else
    record_check "checkout_clean" "git -C \"$CHECKOUT_PATH\" status --porcelain --untracked-files=no" "skipped"
  fi

  if [ "$check_failed" -eq 0 ]; then
    metadata_verified="true"
    reason_code="fork_metadata_verified"
  fi
fi

python3 "$ROOT_DIR/scripts/kolme/contracts/local_fork_sync_metadata_summary.py" "$OUTPUT_JSON" "$MODE" "$overall_status" "$reason_code" "$CHECKOUT_PATH" "$EXPECTED_REMOTE_URL" "$EXPECTED_REF" "$EXPECTED_COMMIT" "$remote_url" "$head_ref" "$head_commit" "$dirty_checkout" "$metadata_verified" "$CHECK_FILE"

echo "status=$overall_status"
echo "sync_mode=$MODE"
echo "reason_code=$reason_code"
echo "metadata_verified=$metadata_verified"
echo "summary_file=$(realpath "$OUTPUT_JSON")"

if [ "$overall_status" != "ok" ]; then
  exit 1
fi
