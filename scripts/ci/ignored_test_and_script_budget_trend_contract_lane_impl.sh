#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_ignored_test_and_script_budget_trend_contract_lane.sh \
  --output-json <path> \
  [--max-runtime-seconds <int>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IGNORED_CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_ignored_test_inventory_drift.sh"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_baseline.json"
METADATA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_metadata.json"
PROMOTION_CRITERIA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_promotion_criteria.json"
COMBINED_TREND_CHECKER="$ROOT_DIR/scripts/ci/check_combined_shell_surface_trend_policy.sh"
COMBINED_THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/combined_shell_surface_trend_thresholds.json"

OUTPUT_JSON=""
MAX_RUNTIME_SECONDS="${KAMN_IGNORED_TEST_SCRIPT_BUDGET_TREND_CONTRACT_MAX_SECONDS:-120}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    --max-runtime-seconds)
      MAX_RUNTIME_SECONDS="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$OUTPUT_JSON" ]; then
  usage >&2
  exit 2
fi

case "$MAX_RUNTIME_SECONDS" in
  ''|*[!0-9]*)
    echo "--max-runtime-seconds must be a non-negative integer" >&2
    exit 2
    ;;
esac

if [ ! -x "$IGNORED_CHECK_SCRIPT" ]; then
  echo "expected ignored-test drift checker to be executable: $IGNORED_CHECK_SCRIPT" >&2
  exit 1
fi
if [ ! -x "$COMBINED_TREND_CHECKER" ]; then
  echo "expected combined shell-surface trend checker to be executable: $COMBINED_TREND_CHECKER" >&2
  exit 1
fi
if [ ! -f "$BASELINE_FILE" ]; then
  echo "expected ignored-test baseline fixture to exist: $BASELINE_FILE" >&2
  exit 1
fi
if [ ! -f "$METADATA_FILE" ]; then
  echo "expected ignored-test metadata fixture to exist: $METADATA_FILE" >&2
  exit 1
fi
if [ ! -f "$PROMOTION_CRITERIA_FILE" ]; then
  echo "expected ignored-test promotion criteria fixture to exist: $PROMOTION_CRITERIA_FILE" >&2
  exit 1
fi
if [ ! -f "$COMBINED_THRESHOLD_FILE" ]; then
  echo "expected combined shell-surface trend threshold fixture to exist: $COMBINED_THRESHOLD_FILE" >&2
  exit 1
fi

mkdir -p "$(dirname "$OUTPUT_JSON")"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

start_epoch="$(date +%s)"

ignored_pass_report="$tmp_dir/ignored-pass-policy.json"
bash "$IGNORED_CHECK_SCRIPT" \
  --baseline-file "$BASELINE_FILE" \
  --metadata-file "$METADATA_FILE" \
  --promotion-criteria-file "$PROMOTION_CRITERIA_FILE" \
  --output-json "$ignored_pass_report" >"$tmp_dir/ignored-pass.out"

if ! grep -q '^status=pass$' "$tmp_dir/ignored-pass.out"; then
  echo "expected ignored-test drift checker to pass for baseline fixtures" >&2
  exit 1
fi

stale_metadata_file="$tmp_dir/ignored-test-metadata.stale.json"
cp "$METADATA_FILE" "$stale_metadata_file"
python3 - "$stale_metadata_file" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
entries = payload.get("ignored_tests")
if not isinstance(entries, list) or not entries:
    raise SystemExit("ignored-test metadata fixture must contain at least one entry")
template = dict(entries[0])
template["source_file"] = "crates/kamn-core/tests/ignored_test_contract_stale_entry.rs"
template["test_name"] = "stale_ignored_test_contract_entry"
entries.append(template)
payload["ignored_tests"] = entries
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
ignored_stale_output="$(
  bash "$IGNORED_CHECK_SCRIPT" \
    --baseline-file "$BASELINE_FILE" \
    --metadata-file "$stale_metadata_file" \
    --promotion-criteria-file "$PROMOTION_CRITERIA_FILE" \
    --output-json "$tmp_dir/ignored-stale-policy.json" 2>&1
)"
ignored_stale_exit_code=$?
set -e

if [ "$ignored_stale_exit_code" -eq 0 ]; then
  echo "expected stale ignored-test metadata contract scenario to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$ignored_stale_output" | grep -q 'ignored_test_metadata_stale_entry'; then
  echo "expected stale ignored-test metadata reason code marker" >&2
  exit 1
fi

combined_within_report="$tmp_dir/combined-shell-surface.within.json"
combined_within_policy="$tmp_dir/combined-shell-surface.within-policy.json"
cat >"$combined_within_report" <<'EOF_WITHIN'
{
  "baseline": {
    "script_count": 100,
    "shell_line_total": 10000,
    "shell_to_rust_ratio": 0.22
  },
  "current": {
    "script_count": 100,
    "shell_line_total": 10000,
    "rust_line_total": 25000,
    "shell_to_rust_ratio": 0.20
  },
  "deltas": {
    "script_count": 0,
    "shell_line_total": 0,
    "shell_to_rust_ratio": -0.02
  },
  "schema_version": "kamn.ci.combined-shell-surface-trend-report.v1",
  "script_budget": {
    "checker_exit_code": 0,
    "pending": [],
    "remediation": "none",
    "status": "pass",
    "violations": [],
    "waived": []
  }
}
EOF_WITHIN

combined_within_output="$(
  bash "$COMBINED_TREND_CHECKER" \
    --report-file "$combined_within_report" \
    --threshold-file "$COMBINED_THRESHOLD_FILE" \
    --output-json "$combined_within_policy"
)"

if ! printf '%s\n' "$combined_within_output" | grep -q '^status=ok$'; then
  echo "expected combined shell-surface within scenario to emit status=ok" >&2
  exit 1
fi
if ! printf '%s\n' "$combined_within_output" | grep -q '^policy_decision=GO$'; then
  echo "expected combined shell-surface within scenario to emit policy_decision=GO" >&2
  exit 1
fi
if ! printf '%s\n' "$combined_within_output" | grep -q '^trend_status=within$'; then
  echo "expected combined shell-surface within scenario to emit trend_status=within" >&2
  exit 1
fi

combined_fail_report="$tmp_dir/combined-shell-surface.fail.json"
combined_fail_policy="$tmp_dir/combined-shell-surface.fail-policy.json"
cp "$combined_within_report" "$combined_fail_report"
python3 - "$combined_fail_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["current"]["shell_to_rust_ratio"] = 0.95
payload["deltas"]["shell_to_rust_ratio"] = 0.50
payload["deltas"]["shell_line_total"] = 200000
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

set +e
combined_fail_output="$(
  bash "$COMBINED_TREND_CHECKER" \
    --report-file "$combined_fail_report" \
    --threshold-file "$COMBINED_THRESHOLD_FILE" \
    --output-json "$combined_fail_policy" 2>&1
)"
combined_fail_exit_code=$?
set -e

if [ "$combined_fail_exit_code" -eq 0 ]; then
  echo "expected combined shell-surface fail scenario to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$combined_fail_output" | grep -q 'combined_shell_surface_shell_line_total_delta_fail_exceeded'; then
  echo "expected combined shell-surface fail scenario to emit shell line delta fail reason code" >&2
  exit 1
fi
if ! printf '%s\n' "$combined_fail_output" | grep -q 'combined_shell_surface_ratio_fail_ceiling_exceeded'; then
  echo "expected combined shell-surface fail scenario to emit ratio fail reason code" >&2
  exit 1
fi

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
status="pass"
reason_code="ignored_test_script_budget_trend_contract_ok"
if [ "$elapsed_seconds" -gt "$MAX_RUNTIME_SECONDS" ]; then
  status="fail"
  reason_code="ignored_test_script_budget_trend_contract_max_runtime_exceeded"
fi

python3 - "$ignored_pass_report" "$combined_within_policy" "$combined_fail_policy" "$OUTPUT_JSON" "$status" "$reason_code" <<'PY'
import json
import sys
from pathlib import Path

ignored_pass = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
combined_within = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))
combined_fail = json.loads(Path(sys.argv[3]).read_text(encoding="utf-8"))
output_path = Path(sys.argv[4])
status = sys.argv[5]
reason_code = sys.argv[6]

summary = {
    "schema_version": "kamn.ci.ignored-test-script-soft-budget-trend-contract-report.v1",
    "status": status,
    "reason_code": reason_code,
    "ignored_inventory_policy_decision": "GO" if ignored_pass.get("status") == "pass" else "NO-GO",
    "ignored_inventory_reason_codes": ignored_pass.get("reason_codes", []),
    "ignored_stale_metadata_reason_contract": "pass",
    "ignored_stale_metadata_reason_code": "ignored_test_metadata_stale_entry",
    "script_trend_within_decision": combined_within.get("policy_decision", ""),
    "script_trend_within_reason_codes": combined_within.get("reason_codes", []),
    "script_trend_fail_decision": combined_fail.get("policy_decision", ""),
    "script_trend_fail_reason_codes": combined_fail.get("reason_codes", []),
    "script_trend_fail_reason_contract": "pass",
}
output_path.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

echo "ignored_test_script_budget_trend_contract_status=$status"
echo "ignored_test_script_budget_trend_contract_ignored_inventory_decision=GO"
echo "ignored_test_script_budget_trend_contract_script_within_decision=GO"
echo "ignored_test_script_budget_trend_contract_script_fail_decision=NO-GO"
echo "ignored_test_script_budget_trend_contract_reason_code=$reason_code"

if [ "$status" != "pass" ]; then
  exit 1
fi
