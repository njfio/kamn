#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_SCRIPT="$ROOT_DIR/scripts/ci/generate_kolme_test_harness_loc_report.sh"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_kolme_test_harness_loc_soft_budget.sh"

output_json=""
input_report_file=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --report-file)
      input_report_file="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [ -z "$output_json" ]; then
  echo "--output-json is required" >&2
  exit 2
fi

tmp_dir="$(mktemp -d)"
cleanup_tmp_dir=true
trap '[ "$cleanup_tmp_dir" = true ] && rm -rf "$tmp_dir"' EXIT

report_file="$input_report_file"
if [ -z "$report_file" ]; then
  report_file="$tmp_dir/kolme-test-harness-loc-report.json"
  bash "$REPORT_SCRIPT" --output-json "$report_file" >/dev/null
elif [ ! -f "$report_file" ]; then
  echo "report file not found: $report_file" >&2
  exit 2
fi

policy_file="$tmp_dir/kolme-test-harness-loc-soft-budget-report.json"

bash "$CHECK_SCRIPT" --report-file "$report_file" --output-json "$policy_file" >/dev/null

output_dir="$(dirname "$output_json")"
mkdir -p "$output_dir"

python3 - "$report_file" "$policy_file" "$output_json" <<'PY'
import json
import sys
from pathlib import Path

report_path = Path(sys.argv[1])
policy_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])

report = json.loads(report_path.read_text(encoding="utf-8"))
policy = json.loads(policy_path.read_text(encoding="utf-8"))

summary = {
    "schema_version": "kamn.ci.kolme-test-harness-loc-trend-report.v1",
    "status": "ok",
    "source_report_file": str(report_path.resolve()),
    "source_policy_file": str(policy_path.resolve()),
    "harness_script_count": report.get("harness_script_count"),
    "harness_shell_line_total": report.get("harness_shell_line_total"),
    "soft_budget_status": policy.get("soft_budget_status"),
    "trend_status": policy.get("trend_status"),
    "policy_decision": policy.get("policy_decision"),
    "review_required": policy.get("review_required"),
    "reason_codes": policy.get("reason_codes", []),
    "delta_harness_script_count": policy.get("delta_harness_script_count"),
    "delta_harness_shell_line_total": policy.get("delta_harness_shell_line_total"),
}

output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

trend_status="$(python3 - "$policy_file" <<'PY'
import json
import sys
from pathlib import Path

policy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(policy.get("trend_status", ""))
PY
)"

policy_decision="$(python3 - "$policy_file" <<'PY'
import json
import sys
from pathlib import Path

policy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(policy.get("policy_decision", ""))
PY
)"

reason_codes="$(python3 - "$policy_file" <<'PY'
import json
import sys
from pathlib import Path

policy = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
codes = policy.get("reason_codes", [])
if not codes:
    print("none")
else:
    print(",".join(codes))
PY
)"

echo "status=ok"
echo "trend_status=$trend_status"
echo "policy_decision=$policy_decision"
echo "reason_codes=$reason_codes"
echo "report_file=$(realpath "$output_json")"
