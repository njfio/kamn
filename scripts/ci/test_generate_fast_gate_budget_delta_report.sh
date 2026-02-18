#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
SCRIPT="$ROOT_DIR/scripts/ci/generate_fast_gate_budget_delta_report.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$SCRIPT" "expected fast-gate budget delta generator to be executable"

CURRENT_JSON="$TMP_DIR/current.json"
BASELINE_ENV="$TMP_DIR/baseline.env"
OUTPUT_JSON="$TMP_DIR/delta.json"

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$CURRENT_JSON" <<'JSON'
{
  "lane": "fast-gate",
  "status": "pass",
  "test_scope": "kolme-local-heavy-contract",
  "elapsed_seconds": 300,
  "runner_minutes": 5
}
JSON

cat >"$BASELINE_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=240
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=50
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=50
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2026-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=36500
ENV

bash "$SCRIPT" \
  --current-json "$CURRENT_JSON" \
  --baseline-file "$BASELINE_ENV" \
  --output-json "$OUTPUT_JSON" >"$TMP_DIR/generate.out"

grep -q '^status=generated$' "$TMP_DIR/generate.out"
grep -q '"schema_version": "kamn.ci.fast-gate-budget-delta-report.v1"' "$OUTPUT_JSON"
grep -q '"elapsed_seconds_delta": 60' "$OUTPUT_JSON"
grep -q '"elapsed_seconds_delta_pct": 25.0' "$OUTPUT_JSON"
grep -q '"runner_minutes_delta": 1' "$OUTPUT_JSON"
grep -q '"runner_minutes_delta_pct": 25.0' "$OUTPUT_JSON"
grep -q '"test_scope": "kolme-local-heavy-contract"' "$OUTPUT_JSON"
grep -q '"local_heavy_sensitive": true' "$OUTPUT_JSON"
grep -q '"local_heavy_scope_class": "contract"' "$OUTPUT_JSON"
grep -q '"local_heavy_sensitive_drift_detected": true' "$OUTPUT_JSON"

cat >"$TMP_DIR/invalid-baseline.env" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=240
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=50
FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON=2026-01-01
FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS=36500
ENV

if bash "$SCRIPT" \
  --current-json "$CURRENT_JSON" \
  --baseline-file "$TMP_DIR/invalid-baseline.env" \
  --output-json "$TMP_DIR/invalid.json" >"$TMP_DIR/invalid.out" 2>&1; then
  echo "expected delta generator to fail for missing threshold config" >&2
  exit 1
fi

grep -q 'FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT is required' "$TMP_DIR/invalid.out"

echo "fast-gate budget delta report generator tests passed."
