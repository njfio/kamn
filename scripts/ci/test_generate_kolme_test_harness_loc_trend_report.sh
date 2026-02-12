#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_test_harness_loc_trend_report.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected Kolme test harness trend report generator to be executable" >&2
  exit 1
fi

default_report="$TMP_DIR/kolme-trend-default.json"
default_output="$(bash "$SCRIPT" --output-json "$default_report")"
if ! printf '%s\n' "$default_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for default Kolme trend report generation path" >&2
  exit 1
fi
if ! printf '%s\n' "$default_output" | grep -q '^trend_status='; then
  echo "expected trend_status marker for default Kolme trend report generation path" >&2
  exit 1
fi
if ! printf '%s\n' "$default_output" | grep -q '^policy_decision='; then
  echo "expected policy_decision marker for default Kolme trend report generation path" >&2
  exit 1
fi

within_input="$TMP_DIR/kolme-trend-within-input.json"
cat >"$within_input" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 62,
  "harness_shell_line_total": 8691
}
EOF_REPORT

within_report="$TMP_DIR/kolme-trend-within.json"
within_output="$(bash "$SCRIPT" --report-file "$within_input" --output-json "$within_report")"
if ! printf '%s\n' "$within_output" | grep -q '^trend_status=within$'; then
  echo "expected trend_status=within for deterministic within-threshold trend report path" >&2
  exit 1
fi
if ! printf '%s\n' "$within_output" | grep -q '^policy_decision=GO$'; then
  echo "expected policy_decision=GO for deterministic within-threshold trend report path" >&2
  exit 1
fi

fail_input="$TMP_DIR/kolme-trend-fail-input.json"
cat >"$fail_input" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 90,
  "harness_shell_line_total": 13000
}
EOF_REPORT

fail_report="$TMP_DIR/kolme-trend-fail.json"
fail_output="$(bash "$SCRIPT" --report-file "$fail_input" --output-json "$fail_report")"
if ! printf '%s\n' "$fail_output" | grep -q '^trend_status=fail$'; then
  echo "expected trend_status=fail for deterministic fail-threshold trend report path" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^policy_decision=NO-GO$'; then
  echo "expected policy_decision=NO-GO for deterministic fail-threshold trend report path" >&2
  exit 1
fi

python3 - "$within_report" "$fail_report" <<'PY'
import json
import sys
from pathlib import Path

for path in sys.argv[1:]:
    payload = json.loads(Path(path).read_text(encoding="utf-8"))
    if payload.get("schema_version") != "kamn.ci.kolme-test-harness-loc-trend-report.v1":
        raise SystemExit("unexpected schema_version in Kolme trend report")
    if "trend_status" not in payload:
        raise SystemExit("missing trend_status in Kolme trend report")
    if "policy_decision" not in payload:
        raise SystemExit("missing policy_decision in Kolme trend report")
PY

echo "Kolme test harness trend report generator tests passed."
