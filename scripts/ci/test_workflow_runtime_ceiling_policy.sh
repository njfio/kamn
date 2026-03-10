#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKFLOWS_DIR="$ROOT_DIR/.github/workflows"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
MAX_TIMEOUT=60

fail() {
  echo "$1" >&2
  exit 1
}

require_file() {
  local path="$1"
  [ -f "$path" ] || fail "missing required file: $path"
}

workflow_has_pr_trigger() {
  local file="$1"
  awk '
    /^on:/ { in_on=1; next }
    in_on && /^[^[:space:]]/ { in_on=0 }
    in_on && $1 ~ /^pull_request:/ { found=1 }
    END { exit found ? 0 : 1 }
  ' "$file"
}

assert_pr_concurrency_policy() {
  local file="$1"
  awk '
    /^concurrency:/ { seen=1; in_block=1; next }
    in_block && /^[^[:space:]]/ { in_block=0 }
    in_block && $1 ~ /^cancel-in-progress:/ && $2 == "true" { cancel=1 }
    END { exit (seen && cancel) ? 0 : 1 }
  ' "$file" || fail "workflow missing top-level PR concurrency cancellation: $file"
}

assert_timeout_policy() {
  local file="$1"
  awk -v limit="$MAX_TIMEOUT" '
    BEGIN { found=0 }
    /^[[:space:]]{4}timeout-minutes:/ {
      found=1
      value=$2 + 0
      if (value > limit) {
        printf("workflow timeout exceeds ceiling (%s > %s): %s\n", $2, limit, FILENAME) > "/dev/stderr"
        exit 2
      }
    }
    END {
      if (!found) {
        printf("workflow missing job timeout-minutes: %s\n", FILENAME) > "/dev/stderr"
        exit 3
      }
    }
  ' "$file" || exit $?
}

assert_strategy_doc_markers() {
  require_file "$STRATEGY_DOC"
  grep -Fq "workflow_runtime_ceiling_minutes=60" "$STRATEGY_DOC" || fail "missing workflow runtime ceiling marker in docs/ci/strategy.md"
  grep -Fq "workflow_pr_concurrency_cancel_in_progress=true" "$STRATEGY_DOC" || fail "missing workflow PR concurrency cancellation marker in docs/ci/strategy.md"
}

main() {
  require_file "$WORKFLOWS_DIR/ci-fast-gate.yml"
  require_file "$WORKFLOWS_DIR/ci-deep-validate.yml"
  require_file "$WORKFLOWS_DIR/ci-supply-chain-advisory.yml"
  require_file "$WORKFLOWS_DIR/e2e-live.yml"
  require_file "$WORKFLOWS_DIR/branch-cleanup.yml"

  local file
  while IFS= read -r file; do
    assert_timeout_policy "$file"
    if workflow_has_pr_trigger "$file"; then
      assert_pr_concurrency_policy "$file"
    fi
  done < <(find "$WORKFLOWS_DIR" -maxdepth 1 -name '*.yml' | sort)

  assert_strategy_doc_markers
}

main "$@"
