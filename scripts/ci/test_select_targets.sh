#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/select_targets.sh"

extract_output() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { sub($1 "=",""); print; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

run_selector() {
  local changed_files="$1"
  CI_CHANGED_FILES="$changed_files" GITHUB_BASE_REF=__missing__ bash "$SCRIPT"
}

docs_output="$(run_selector $'docs/foundation/ci-caching-parallelism.md')"
assert_eq "$(extract_output "$docs_output" "docs_only")" "true" "docs_only selection mismatch"
assert_eq "$(extract_output "$docs_output" "run_rust")" "false" "docs_only should not run rust"
assert_eq "$(extract_output "$docs_output" "test_scope")" "none" "docs_only should keep none scope"

critical_output="$(run_selector $'.github/workflows/ci-fast-gate.yml')"
assert_eq "$(extract_output "$critical_output" "run_rust")" "true" "workflow changes must run rust"
assert_eq "$(extract_output "$critical_output" "test_scope")" "full" "workflow changes must use full scope"

unknown_output="$(run_selector $'config/runtime-policy.json')"
assert_eq "$(extract_output "$unknown_output" "run_rust")" "true" "unknown paths must run rust fallback"
assert_eq "$(extract_output "$unknown_output" "test_scope")" "full" "unknown paths must use full fallback"

targeted_output="$(run_selector $'crates/kamn-core/src/bridge_adapter.rs')"
assert_eq "$(extract_output "$targeted_output" "run_rust")" "true" "rust path should run rust"
assert_eq "$(extract_output "$targeted_output" "test_scope")" "targeted" "crate path should be targeted"

test_cmd="$(extract_output "$targeted_output" "test_cmd")"
if ! printf '%s\n' "$test_cmd" | grep -q "run_cargo_test_with_quarantine.sh"; then
  echo "targeted test command must use quarantine wrapper" >&2
  exit 1
fi

echo "select_targets matrix regression tests passed."
