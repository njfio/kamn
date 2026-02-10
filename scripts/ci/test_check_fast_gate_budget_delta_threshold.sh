#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_fast_gate_budget_delta_threshold.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected fast-gate budget delta threshold checker to be executable" >&2
  exit 1
fi

THRESHOLD_ENV="$TMP_DIR/threshold.env"
PASS_REPORT="$TMP_DIR/pass.json"
FAIL_REPORT="$TMP_DIR/fail.json"
WAIVER_JSON="$TMP_DIR/waiver.json"
EXPIRED_WAIVER_JSON="$TMP_DIR/expired-waiver.json"
TAMPERED_REPORT="$TMP_DIR/tampered.json"

cat >"$THRESHOLD_ENV" <<'ENV'
FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS=230
FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES=4
FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT=20
FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT=20
ENV

cat >"$PASS_REPORT" <<'JSON'
{
  "schema_version": "kamn.ci.fast-gate-budget-delta-report.v1",
  "lane": "fast-gate",
  "variance": {
    "elapsed_seconds_delta": 10,
    "elapsed_seconds_delta_pct": 4.35,
    "runner_minutes_delta": 0,
    "runner_minutes_delta_pct": 0.0
  }
}
JSON

cat >"$FAIL_REPORT" <<'JSON'
{
  "schema_version": "kamn.ci.fast-gate-budget-delta-report.v1",
  "lane": "fast-gate",
  "variance": {
    "elapsed_seconds_delta": 70,
    "elapsed_seconds_delta_pct": 30.43,
    "runner_minutes_delta": 2,
    "runner_minutes_delta_pct": 50.0
  }
}
JSON

cat >"$WAIVER_JSON" <<'JSON'
{
  "reason": "Temporary migration overhead while lane framework settles",
  "expires_on": "2099-12-31",
  "allow_metrics": [
    "elapsed_seconds_delta_pct",
    "runner_minutes_delta_pct"
  ]
}
JSON

cat >"$EXPIRED_WAIVER_JSON" <<'JSON'
{
  "reason": "Expired override",
  "expires_on": "2000-01-01",
  "allow_metrics": [
    "elapsed_seconds_delta_pct",
    "runner_minutes_delta_pct"
  ]
}
JSON

cat >"$TAMPERED_REPORT" <<'JSON'
{
  "schema_version": "kamn.ci.fast-gate-budget-delta-report.v0",
  "lane": "fast-gate",
  "variance": {}
}
JSON

start_epoch="$(date +%s)"
bash "$SCRIPT" \
  --report-json "$PASS_REPORT" \
  --threshold-file "$THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" >"$TMP_DIR/pass.out"
elapsed_seconds=$(( $(date +%s) - start_epoch ))
if [ "$elapsed_seconds" -gt 2 ]; then
  echo "expected threshold checker overhead <= 2s for pass path" >&2
  exit 1
fi
grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^waived=false$' "$TMP_DIR/pass.out"

if bash "$SCRIPT" \
  --report-json "$FAIL_REPORT" \
  --threshold-file "$THRESHOLD_ENV" \
  --waiver-file "$TMP_DIR/missing-waiver.json" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected threshold checker to fail when violations have no waiver" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/fail.out"
grep -q 'violations=elapsed_seconds_delta_pct,runner_minutes_delta_pct' "$TMP_DIR/fail.out"

bash "$SCRIPT" \
  --report-json "$FAIL_REPORT" \
  --threshold-file "$THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" >"$TMP_DIR/waived.out"
grep -q '^status=pass$' "$TMP_DIR/waived.out"
grep -q '^waived=true$' "$TMP_DIR/waived.out"

if bash "$SCRIPT" \
  --report-json "$FAIL_REPORT" \
  --threshold-file "$THRESHOLD_ENV" \
  --waiver-file "$EXPIRED_WAIVER_JSON" >"$TMP_DIR/expired.out" 2>&1; then
  echo "expected threshold checker to fail for expired waiver" >&2
  exit 1
fi
grep -q 'waiver expired' "$TMP_DIR/expired.out"

if bash "$SCRIPT" \
  --report-json "$TAMPERED_REPORT" \
  --threshold-file "$THRESHOLD_ENV" \
  --waiver-file "$WAIVER_JSON" >"$TMP_DIR/tampered.out" 2>&1; then
  echo "expected threshold checker to fail for tampered report schema" >&2
  exit 1
fi
grep -q 'unexpected schema_version' "$TMP_DIR/tampered.out"

echo "fast-gate budget delta threshold checker tests passed."
