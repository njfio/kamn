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
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$within_input" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 62,
  "harness_shell_line_total": 8691
}
EOF_REPORT

command_surface_within_report="$TMP_DIR/kolme-command-surface-within-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$command_surface_within_report" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.script-surface-budget-report.v1",
  "status": "pass",
  "metrics": {
    "script_count": 56,
    "shell_line_total": 12958,
    "duplicate_basename": 0,
    "duplicate_content": 0
  },
  "deltas": {
    "script_count": 0,
    "shell_line_total": 0,
    "duplicate_basename": 0,
    "duplicate_content": 0
  }
}
EOF_REPORT

within_report="$TMP_DIR/kolme-trend-within.json"
within_output="$(bash "$SCRIPT" --report-file "$within_input" --command-surface-report-file "$command_surface_within_report" --output-json "$within_report")"
if ! printf '%s\n' "$within_output" | grep -q '^trend_status=within$'; then
  echo "expected trend_status=within for deterministic within-threshold trend report path" >&2
  exit 1
fi
if ! printf '%s\n' "$within_output" | grep -q '^policy_decision=GO$'; then
  echo "expected policy_decision=GO for deterministic within-threshold trend report path" >&2
  exit 1
fi
if ! printf '%s\n' "$within_output" | grep -q '^command_surface_trend_status=within$'; then
  echo "expected command_surface_trend_status=within for deterministic within-threshold trend report path" >&2
  exit 1
fi
if ! printf '%s\n' "$within_output" | grep -q '^command_surface_policy_decision=GO$'; then
  echo "expected command_surface_policy_decision=GO for deterministic within-threshold trend report path" >&2
  exit 1
fi

fail_input="$TMP_DIR/kolme-trend-fail-input.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$fail_input" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.test-harness-loc-report.v1",
  "harness_script_count": 90,
  "harness_shell_line_total": 13000
}
EOF_REPORT

fail_report="$TMP_DIR/kolme-trend-fail.json"
fail_output="$(bash "$SCRIPT" --report-file "$fail_input" --command-surface-report-file "$command_surface_within_report" --output-json "$fail_report")"
if ! printf '%s\n' "$fail_output" | grep -q '^trend_status=fail$'; then
  echo "expected trend_status=fail for deterministic fail-threshold trend report path" >&2
  exit 1
fi
if ! printf '%s\n' "$fail_output" | grep -q '^policy_decision=NO-GO$'; then
  echo "expected policy_decision=NO-GO for deterministic fail-threshold trend report path" >&2
  exit 1
fi

command_surface_warn_report="$TMP_DIR/kolme-command-surface-warn-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$command_surface_warn_report" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.script-surface-budget-report.v1",
  "status": "pass",
  "metrics": {
    "script_count": 60,
    "shell_line_total": 14458,
    "duplicate_basename": 0,
    "duplicate_content": 0
  },
  "deltas": {
    "script_count": 4,
    "shell_line_total": 1500,
    "duplicate_basename": 0,
    "duplicate_content": 0
  }
}
EOF_REPORT

command_surface_warn_output="$(
  bash "$SCRIPT" \
    --report-file "$within_input" \
    --command-surface-report-file "$command_surface_warn_report" \
    --output-json "$TMP_DIR/kolme-command-surface-warn-trend.json"
)"
if ! printf '%s\n' "$command_surface_warn_output" | grep -q '^command_surface_trend_status=warn$'; then
  echo "expected command_surface_trend_status=warn for deterministic warn command-surface trend path" >&2
  exit 1
fi
if ! printf '%s\n' "$command_surface_warn_output" | grep -q '^command_surface_policy_decision=WARN$'; then
  echo "expected command_surface_policy_decision=WARN for deterministic warn command-surface trend path" >&2
  exit 1
fi
if ! printf '%s\n' "$command_surface_warn_output" | grep -q '^policy_decision=WARN$'; then
  echo "expected combined policy_decision=WARN for deterministic warn command-surface trend path" >&2
  exit 1
fi

command_surface_fail_report="$TMP_DIR/kolme-command-surface-fail-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$command_surface_fail_report" <<'EOF_REPORT'
{
  "schema_version": "kamn.ci.script-surface-budget-report.v1",
  "status": "pass",
  "metrics": {
    "script_count": 66,
    "shell_line_total": 15358,
    "duplicate_basename": 0,
    "duplicate_content": 0
  },
  "deltas": {
    "script_count": 10,
    "shell_line_total": 2400,
    "duplicate_basename": 0,
    "duplicate_content": 0
  }
}
EOF_REPORT

command_surface_fail_output="$(
  bash "$SCRIPT" \
    --report-file "$within_input" \
    --command-surface-report-file "$command_surface_fail_report" \
    --output-json "$TMP_DIR/kolme-command-surface-fail-trend.json"
)"
if ! printf '%s\n' "$command_surface_fail_output" | grep -q '^command_surface_trend_status=fail$'; then
  echo "expected command_surface_trend_status=fail for deterministic fail command-surface trend path" >&2
  exit 1
fi
if ! printf '%s\n' "$command_surface_fail_output" | grep -q '^command_surface_policy_decision=NO-GO$'; then
  echo "expected command_surface_policy_decision=NO-GO for deterministic fail command-surface trend path" >&2
  exit 1
fi

set +e
command_surface_enforced_fail_output="$(
  bash "$SCRIPT" \
    --report-file "$within_input" \
    --command-surface-report-file "$command_surface_fail_report" \
    --enforce-command-surface-fail \
    --output-json "$TMP_DIR/kolme-command-surface-fail-enforced-trend.json" 2>&1
)"
command_surface_enforced_fail_code=$?
set -e
if [ "$command_surface_enforced_fail_code" -eq 0 ]; then
  echo "expected enforce-command-surface-fail to return non-zero for command-surface fail trend status" >&2
  exit 1
fi
if ! printf '%s\n' "$command_surface_enforced_fail_output" | grep -q '^status=fail$'; then
  echo "expected status=fail marker for enforce-command-surface-fail path" >&2
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
    if "command_surface_trend_status" not in payload:
        raise SystemExit("missing command_surface_trend_status in Kolme trend report")
    if "command_surface_policy_decision" not in payload:
        raise SystemExit("missing command_surface_policy_decision in Kolme trend report")
PY

echo "Kolme test harness trend report generator tests passed."
