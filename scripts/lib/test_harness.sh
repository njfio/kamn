#!/usr/bin/env bash

# Shared shell test harness helpers for deterministic script contract tests.

if [[ "${KAMN_TEST_HARNESS_SH_LOADED:-0}" -eq 1 ]]; then
  return 0
fi
KAMN_TEST_HARNESS_SH_LOADED=1

test_harness_tmp_dir=""

test_harness_setup() {
  if [[ -n "$test_harness_tmp_dir" && -d "$test_harness_tmp_dir" ]]; then
    return 0
  fi
  test_harness_tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$test_harness_tmp_dir"' EXIT
}

test_harness_require_file() {
  local file_path="$1"
  local message="$2"
  if [[ ! -f "$file_path" ]]; then
    echo "$message" >&2
    return 1
  fi
}

test_harness_require_executable() {
  local file_path="$1"
  local message="$2"
  if [[ ! -x "$file_path" ]]; then
    echo "$message" >&2
    return 1
  fi
}

test_harness_assert_file_contains() {
  local file_path="$1"
  local pattern="$2"
  local message="$3"
  if ! grep -q -- "$pattern" "$file_path"; then
    echo "$message" >&2
    return 1
  fi
}

test_harness_assert_file_contains_fixed() {
  local file_path="$1"
  local text="$2"
  local message="$3"
  if ! grep -Fq -- "$text" "$file_path"; then
    echo "$message" >&2
    return 1
  fi
}
