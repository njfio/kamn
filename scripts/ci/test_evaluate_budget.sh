#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BUDGET_FILE="$ROOT_DIR/.ci/ci-budget.env"
SCRIPT="$ROOT_DIR/scripts/ci/evaluate_budget.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

run_expect_status() {
  local name="$1"
  local expected_status="$2"
  shift 2
  local json_path="$TMP_DIR/${name}.json"
  "$SCRIPT" "$@" --output-json "$json_path" >"$TMP_DIR/${name}.out"
  grep -q "\"status\": \"$expected_status\"" "$json_path"
}

run_expect_failure() {
  local name="$1"
  shift
  local json_path="$TMP_DIR/${name}.json"
  if "$SCRIPT" "$@" --output-json "$json_path" >"$TMP_DIR/${name}.out" 2>&1; then
    echo "Expected failure for $name but command succeeded" >&2
    exit 1
  fi
  grep -q 'status=fail' "$TMP_DIR/${name}.out"
}

run_expect_status fast_pass pass --budget-file "$BUDGET_FILE" --lane fast-gate --elapsed-seconds 180 --test-scope targeted --changed-files 4 --job-count 1
run_expect_status fast_warn warn --budget-file "$BUDGET_FILE" --lane fast-gate --elapsed-seconds 820 --test-scope full --changed-files 10 --job-count 1
run_expect_failure fast_fail --budget-file "$BUDGET_FILE" --lane fast-gate --elapsed-seconds 901 --test-scope full --changed-files 12 --job-count 1
run_expect_status deep_pass pass --budget-file "$BUDGET_FILE" --lane deep-validate --elapsed-seconds 900 --test-scope full --changed-files 0 --job-count 1
run_expect_status deep_warn warn --budget-file "$BUDGET_FILE" --lane deep-validate --elapsed-seconds 7000 --test-scope full --changed-files 0 --job-count 1

echo "CI budget evaluator tests passed."
