#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

source "$ROOT_DIR/scripts/lib/common.sh"

assert_eq "ok" "ok" "assert_eq should compare equal values"
assert_contains "hello world" "world" "assert_contains should find substring"

output="$(
  printf '%s\n' "status=pass" "final_decision=GO"
)"
assert_eq "$(extract_value "$output" "status")" "pass" "extract_value should parse key=value output"
assert_eq "$(extract_value "$output" "final_decision")" "GO" "extract_value should parse second key=value output"

FALLBACK_REASON_TAXONOMY_VERSION="kamn.framework.fallback-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV="reason_a,reason_b"
fallback_output="$(
  emit_fallback_error "reason_a" "detail" 2>&1
)"
assert_contains "$fallback_output" "dispatch_status=fail" "emit_fallback_error should emit fail status"
assert_contains "$fallback_output" "fallback_reason_code=reason_a" "emit_fallback_error should emit reason code"
assert_contains "$fallback_output" "fallback_reason_detail=detail" "emit_fallback_error should emit reason detail"

echo "common shell library tests passed."
