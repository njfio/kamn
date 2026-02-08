#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_performance_thresholds.sh"
PROFILE_FILE="$ROOT_DIR/.ci/performance-targets.env"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_REPORT="$TMP_DIR/pass.json"
FAIL_REPORT="$TMP_DIR/fail.json"
INVALID_REPORT="$TMP_DIR/invalid.json"

cat >"$PASS_REPORT" <<'JSON'
{
  "latency_p50_ms": 88,
  "latency_p99_ms": 300,
  "throughput_tps": 12500,
  "availability_pct": 99.95
}
JSON

cat >"$FAIL_REPORT" <<'JSON'
{
  "latency_p50_ms": 101,
  "latency_p99_ms": 540,
  "throughput_tps": 9800,
  "availability_pct": 99.7
}
JSON

cat >"$INVALID_REPORT" <<'JSON'
{
  "latency_p50_ms": 90,
  "throughput_tps": 12000
}
JSON

bash "$SCRIPT" --report-json "$PASS_REPORT" --profile-file "$PROFILE_FILE" --lane smoke >"$TMP_DIR/pass.out"
grep -q 'status=pass' "$TMP_DIR/pass.out"

if bash "$SCRIPT" --report-json "$FAIL_REPORT" --profile-file "$PROFILE_FILE" --lane smoke >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected failure for threshold breach report" >&2
  exit 1
fi
grep -q 'status=fail' "$TMP_DIR/fail.out"
grep -q 'latency_p50_ms' "$TMP_DIR/fail.out"
grep -q 'throughput_tps' "$TMP_DIR/fail.out"

if bash "$SCRIPT" --report-json "$INVALID_REPORT" --profile-file "$PROFILE_FILE" --lane smoke >"$TMP_DIR/invalid.out" 2>&1; then
  echo "expected failure for invalid report schema" >&2
  exit 1
fi
grep -q 'missing required metric' "$TMP_DIR/invalid.out"

echo "performance threshold gate tests passed."
