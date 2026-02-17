#!/usr/bin/env bash

# Shared shell helpers for KAMN scripts.
#
# This file is sourced by shell scripts. Keep functions deterministic and
# side-effect free unless explicitly documented.

if [[ "${KAMN_COMMON_SH_LOADED:-0}" -eq 1 ]]; then
  return 0
fi
KAMN_COMMON_SH_LOADED=1

KAMN_ROOT="${KAMN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [[ "$actual" != "$expected" ]]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    return 1
  fi
}

assert_contains() {
  local haystack="$1"
  local needle="$2"
  local message="$3"
  if [[ "$haystack" != *"$needle"* ]]; then
    echo "$message: expected to contain '$needle'" >&2
    return 1
  fi
}

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

emit_fallback_error() {
  local reason_code="$1"
  local reason_detail="$2"
  local taxonomy_version="${FALLBACK_REASON_TAXONOMY_VERSION:-kamn.framework.fallback-reason-taxonomy.v1}"
  local reason_codes_csv="${FALLBACK_REASON_CODES_CSV:-$reason_code}"
  echo "dispatch_status=fail" >&2
  echo "fallback_reason_taxonomy_version=$taxonomy_version" >&2
  echo "fallback_reason_codes_csv=$reason_codes_csv" >&2
  echo "fallback_reason_code=$reason_code" >&2
  echo "fallback_reason_detail=$reason_detail" >&2
}

emit_json_object() {
  if (( $# % 2 != 0 )); then
    echo "emit_json_object requires key/value pairs" >&2
    return 1
  fi
  python3 - "$@" <<'PY'
import json
import sys

args = sys.argv[1:]
payload = {}
for idx in range(0, len(args), 2):
    payload[args[idx]] = args[idx + 1]
json.dump(payload, sys.stdout, sort_keys=True)
sys.stdout.write("\n")
PY
}

write_json_file() {
  local output_file="$1"
  mkdir -p "$(dirname "$output_file")"
  cat >"$output_file"
}

write_json_object() {
  local output_file="$1"
  shift
  emit_json_object "$@" >"$output_file"
}

write_decision_json() {
  local output_file="$1"
  local final_decision="$2"
  local reason_taxonomy_version="$3"
  local reason_codes_csv="$4"
  shift 4
  write_json_object \
    "$output_file" \
    "final_decision" "$final_decision" \
    "reason_taxonomy_version" "$reason_taxonomy_version" \
    "reason_codes_csv" "$reason_codes_csv" \
    "$@"
}
