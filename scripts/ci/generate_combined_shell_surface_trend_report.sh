#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_script_duplication_budget.py"
DEFAULT_BUDGET_FILE="$ROOT_DIR/.ci/script-surface-budget.env"
DEFAULT_SCRIPT_BASELINE_FILE="$ROOT_DIR/.ci/script-surface-baseline.env"
DEFAULT_COMBINED_BASELINE_FILE="$ROOT_DIR/fixtures/ci/combined_shell_surface_trend_baseline.json"

output_json=""
scripts_root="$ROOT_DIR/scripts"
rust_root="$ROOT_DIR"
budget_file="$DEFAULT_BUDGET_FILE"
script_baseline_file="$DEFAULT_SCRIPT_BASELINE_FILE"
combined_baseline_file="$DEFAULT_COMBINED_BASELINE_FILE"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --scripts-root)
      scripts_root="${2:-}"
      shift 2
      ;;
    --rust-root)
      rust_root="${2:-}"
      shift 2
      ;;
    --budget-file)
      budget_file="${2:-}"
      shift 2
      ;;
    --script-baseline-file)
      script_baseline_file="${2:-}"
      shift 2
      ;;
    --combined-baseline-file)
      combined_baseline_file="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  echo "--output-json is required" >&2
  exit 2
fi

if [[ ! -f "$budget_file" ]]; then
  echo "budget file not found: $budget_file" >&2
  exit 2
fi
if [[ ! -f "$script_baseline_file" ]]; then
  echo "script baseline file not found: $script_baseline_file" >&2
  exit 2
fi
if [[ ! -f "$combined_baseline_file" ]]; then
  echo "combined baseline file not found: $combined_baseline_file" >&2
  exit 2
fi
if [[ ! -d "$scripts_root" ]]; then
  echo "scripts root not found: $scripts_root" >&2
  exit 2
fi
if [[ ! -d "$rust_root" ]]; then
  echo "rust root not found: $rust_root" >&2
  exit 2
fi

tmp_dir="$(mktemp -d)"
cleanup_tmp_dir=true
trap '[ "$cleanup_tmp_dir" = true ] && rm -rf "$tmp_dir"' EXIT

script_budget_report="$tmp_dir/script-surface-budget-report.json"
set +e
python3 "$CHECK_SCRIPT" \
  --scripts-root "$scripts_root" \
  --budget-file "$budget_file" \
  --baseline-file "$script_baseline_file" \
  --output-json "$script_budget_report" >/dev/null
script_budget_exit_code=$?
set -e

if [[ ! -f "$script_budget_report" ]]; then
  echo "script budget checker did not emit report: $script_budget_report" >&2
  exit 1
fi

mkdir -p "$(dirname "$output_json")"

python3 - "$script_budget_report" "$combined_baseline_file" "$output_json" "$script_budget_exit_code" "$rust_root" <<'PY'
import datetime as dt
import json
import sys
from pathlib import Path

script_budget_path = Path(sys.argv[1])
combined_baseline_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])
script_budget_exit_code = int(sys.argv[4])
rust_root = Path(sys.argv[5])

script_budget = json.loads(script_budget_path.read_text(encoding="utf-8"))
baseline = json.loads(combined_baseline_path.read_text(encoding="utf-8"))

if baseline.get("schema_version") != "kamn.ci.combined-shell-surface-trend-baseline.v1":
    raise SystemExit("combined baseline schema mismatch")

metrics = script_budget.get("metrics", {})
if not isinstance(metrics, dict):
    raise SystemExit("script budget report missing metrics object")

script_count = int(metrics.get("script_count", 0))
shell_line_total = int(metrics.get("shell_line_total", 0))

rust_line_total = 0
for path in rust_root.rglob("*.rs"):
    rel_parts = set(path.parts)
    if ".git" in rel_parts or "target" in rel_parts:
        continue
    rust_line_total += sum(1 for _ in path.open("r", encoding="utf-8", errors="ignore"))

if rust_line_total <= 0:
    raise SystemExit("rust_line_total must be positive")

shell_to_rust_ratio = round(shell_line_total / rust_line_total, 6)
baseline_script_count = int(baseline.get("script_count", 0))
baseline_shell_line_total = int(baseline.get("shell_line_total", 0))
baseline_rust_line_total = int(baseline.get("rust_line_total", 0))
baseline_shell_to_rust_ratio = float(baseline.get("shell_to_rust_ratio", 0.0))

delta_script_count = script_count - baseline_script_count
delta_shell_line_total = shell_line_total - baseline_shell_line_total
delta_rust_line_total = rust_line_total - baseline_rust_line_total
delta_shell_to_rust_ratio = round(shell_to_rust_ratio - baseline_shell_to_rust_ratio, 6)

report = {
    "schema_version": "kamn.ci.combined-shell-surface-trend-report.v1",
    "status": "generated",
    "generated_at_utc": dt.datetime.now(tz=dt.timezone.utc).isoformat(),
    "current": {
        "script_count": script_count,
        "shell_line_total": shell_line_total,
        "rust_line_total": rust_line_total,
        "shell_to_rust_ratio": shell_to_rust_ratio,
    },
    "baseline": {
        "script_count": baseline_script_count,
        "shell_line_total": baseline_shell_line_total,
        "rust_line_total": baseline_rust_line_total,
        "shell_to_rust_ratio": baseline_shell_to_rust_ratio,
    },
    "deltas": {
        "script_count": delta_script_count,
        "shell_line_total": delta_shell_line_total,
        "rust_line_total": delta_rust_line_total,
        "shell_to_rust_ratio": delta_shell_to_rust_ratio,
    },
    "script_budget": {
        "status": script_budget.get("status", "fail"),
        "checker_exit_code": script_budget_exit_code,
        "violations": script_budget.get("violations", []),
        "waived": script_budget.get("waived", []),
        "pending": script_budget.get("pending", []),
        "remediation": script_budget.get("remediation", "none"),
    },
}

output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

print("status=generated")
print(f"script_count={script_count}")
print(f"shell_line_total={shell_line_total}")
print(f"rust_line_total={rust_line_total}")
print(f"shell_to_rust_ratio={shell_to_rust_ratio}")
print(f"delta_script_count={delta_script_count}")
print(f"delta_shell_line_total={delta_shell_line_total}")
print(f"delta_rust_line_total={delta_rust_line_total}")
print(f"delta_shell_to_rust_ratio={delta_shell_to_rust_ratio}")
print(f"script_budget_status={report['script_budget']['status']}")
print(f"script_budget_checker_exit_code={script_budget_exit_code}")
PY
