#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECK_SCRIPT="$ROOT_DIR/scripts/ci/check_script_duplication_budget.py"
DEFAULT_BUDGET_FILE="$ROOT_DIR/.ci/script-surface-budget.env"
DEFAULT_SCRIPT_BASELINE_FILE="$ROOT_DIR/.ci/script-surface-baseline.env"
DEFAULT_COMBINED_BASELINE_FILE="$ROOT_DIR/fixtures/ci/combined_shell_surface_trend_baseline.json"
DEFAULT_GOVERNANCE_STRUCTURAL_COUPLING_POLICY_FILE="$ROOT_DIR/docs/review/governance-structural-coupling.policy"
DEFAULT_REVIEW_DOCS_ROOT="$ROOT_DIR/docs/review"

output_json=""
scripts_root="$ROOT_DIR/scripts"
rust_root="$ROOT_DIR"
budget_file="$DEFAULT_BUDGET_FILE"
script_baseline_file="$DEFAULT_SCRIPT_BASELINE_FILE"
combined_baseline_file="$DEFAULT_COMBINED_BASELINE_FILE"
governance_policy_file="$DEFAULT_GOVERNANCE_STRUCTURAL_COUPLING_POLICY_FILE"
review_docs_root="$DEFAULT_REVIEW_DOCS_ROOT"

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
    --governance-policy-file)
      governance_policy_file="${2:-}"
      shift 2
      ;;
    --review-docs-root)
      review_docs_root="${2:-}"
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
if [[ ! -f "$governance_policy_file" ]]; then
  echo "governance structural-coupling policy file not found: $governance_policy_file" >&2
  exit 2
fi
if [[ ! -d "$review_docs_root" ]]; then
  echo "review docs root not found: $review_docs_root" >&2
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

python3 - "$script_budget_report" "$combined_baseline_file" "$output_json" "$script_budget_exit_code" "$rust_root" "$governance_policy_file" "$review_docs_root" <<'PY'
import datetime as dt
import json
import re
import sys
from pathlib import Path

script_budget_path = Path(sys.argv[1])
combined_baseline_path = Path(sys.argv[2])
output_path = Path(sys.argv[3])
script_budget_exit_code = int(sys.argv[4])
rust_root = Path(sys.argv[5])
governance_policy_path = Path(sys.argv[6])
review_docs_root = Path(sys.argv[7])

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


def parse_policy(policy_path: Path) -> dict:
    policy_values: dict[str, str] = {}
    for raw_line in policy_path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        policy_values[key.strip()] = value.strip()
    target_ratio_raw = policy_values.get("review_governance_structural_coupling_target_ratio_max")
    status_within = policy_values.get("review_governance_structural_coupling_status_within")
    status_over = policy_values.get("review_governance_structural_coupling_status_over")
    if not target_ratio_raw or not status_within or not status_over:
        raise SystemExit("governance structural-coupling policy missing required keys")
    try:
        target_ratio_max = float(target_ratio_raw)
    except ValueError as error:
        raise SystemExit(f"governance structural-coupling policy target ratio invalid: {error}") from error
    if target_ratio_max <= 0:
        raise SystemExit("governance structural-coupling policy target ratio must be positive")
    return {
        "target_ratio_max": target_ratio_max,
        "status_within": status_within,
        "status_over": status_over,
    }


def parse_latest_governance_markers(docs_root: Path) -> dict:
    file_pattern = re.compile(r"gaps-and-issues-r(\d+)\.md$")
    marker_pattern = re.compile(
        r"^\s*-\s*(r\d+_review_governance_structural_coupling_[a-z0-9_]+)=(.+?)\s*$",
        re.MULTILINE,
    )
    required_suffixes = [
        "non_merge_commit_count",
        "governance_commit_count",
        "governance_commit_ratio",
        "budget_status",
    ]
    candidates: list[tuple[int, dict[str, str], str]] = []
    for path in sorted(docs_root.glob("gaps-and-issues-r*.md")):
        match = file_pattern.search(path.name)
        if not match:
            continue
        release = int(match.group(1))
        marker_prefix = f"r{release}_review_governance_structural_coupling_"
        markers: dict[str, str] = {}
        content = path.read_text(encoding="utf-8", errors="ignore")
        for marker_match in marker_pattern.finditer(content):
            marker_key = marker_match.group(1).strip()
            marker_value = marker_match.group(2).strip()
            if marker_key.startswith(marker_prefix):
                markers[marker_key] = marker_value
        if all(f"{marker_prefix}{suffix}" in markers for suffix in required_suffixes):
            candidates.append((release, markers, path.name))
    if not candidates:
        return {
            "status": "markers_missing",
            "release": "unknown",
            "source_review_doc": "unknown",
            "target_ratio_max_next_release_marker": "unknown",
            "non_merge_commit_count": "unknown",
            "governance_commit_count": "unknown",
            "governance_commit_ratio": "unknown",
            "computed_governance_commit_ratio": "unknown",
            "ratio_marker_delta": "unknown",
            "delta_to_target_ratio_max": "unknown",
            "budget_status_marker": "unknown",
            "mitigation_issue_marker": "none",
        }
    release, markers, source_doc = max(candidates, key=lambda item: item[0])
    marker_prefix = f"r{release}_review_governance_structural_coupling_"
    non_merge_commit_count = int(markers[f"{marker_prefix}non_merge_commit_count"])
    governance_commit_count = int(markers[f"{marker_prefix}governance_commit_count"])
    governance_commit_ratio = float(markers[f"{marker_prefix}governance_commit_ratio"])
    target_ratio_marker_raw = markers.get(f"{marker_prefix}target_ratio_max_next_release", "unknown")
    target_ratio_marker = (
        float(target_ratio_marker_raw) if target_ratio_marker_raw != "unknown" else "unknown"
    )
    budget_status_marker = markers[f"{marker_prefix}budget_status"]
    mitigation_issue_marker = markers.get(f"{marker_prefix}mitigation_issue", "none")

    if non_merge_commit_count <= 0:
        computed_governance_commit_ratio = "unknown"
        ratio_marker_delta = "unknown"
        status = "marker_count_invalid"
    else:
        computed_governance_commit_ratio = round(
            governance_commit_count / non_merge_commit_count, 4
        )
        ratio_marker_delta = round(
            governance_commit_ratio - computed_governance_commit_ratio,
            6,
        )
        if governance_commit_count < 0 or governance_commit_count > non_merge_commit_count:
            status = "marker_count_invalid"
        elif abs(ratio_marker_delta) > 0.001:
            status = "marker_ratio_mismatch"
        else:
            status = "markers_loaded"
    return {
        "status": status,
        "release": release,
        "source_review_doc": source_doc,
        "target_ratio_max_next_release_marker": target_ratio_marker,
        "non_merge_commit_count": non_merge_commit_count,
        "governance_commit_count": governance_commit_count,
        "governance_commit_ratio": governance_commit_ratio,
        "computed_governance_commit_ratio": computed_governance_commit_ratio,
        "ratio_marker_delta": ratio_marker_delta,
        "delta_to_target_ratio_max": "unknown",
        "budget_status_marker": budget_status_marker,
        "mitigation_issue_marker": mitigation_issue_marker,
    }


governance_policy = parse_policy(governance_policy_path)
governance_markers = parse_latest_governance_markers(review_docs_root)
governance_status = governance_markers["status"]
target_ratio_max = governance_policy["target_ratio_max"]
delta_to_target_ratio_max = "unknown"
governance_ratio_value = governance_markers.get("governance_commit_ratio")
if isinstance(governance_ratio_value, (int, float)):
    delta_to_target_ratio_max = round(governance_ratio_value - target_ratio_max, 6)

if governance_status == "markers_loaded":
    if governance_ratio_value <= target_ratio_max + 0.001:
        governance_status = "within_target"
    elif (
        governance_markers.get("budget_status_marker") == governance_policy["status_over"]
        and str(governance_markers.get("mitigation_issue_marker", "")).strip() not in ("", "none")
    ):
        governance_status = "reduction_contract_active"
    else:
        governance_status = "over_target_unmitigated"

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
    "governance_structural_coupling": {
        "status": governance_status,
        "release": governance_markers.get("release", "unknown"),
        "source_review_doc": governance_markers.get("source_review_doc", "unknown"),
        "target_ratio_max": target_ratio_max,
        "target_ratio_max_next_release_marker": governance_markers.get(
            "target_ratio_max_next_release_marker", "unknown"
        ),
        "non_merge_commit_count": governance_markers.get("non_merge_commit_count", "unknown"),
        "governance_commit_count": governance_markers.get("governance_commit_count", "unknown"),
        "governance_commit_ratio": governance_markers.get("governance_commit_ratio", "unknown"),
        "computed_governance_commit_ratio": governance_markers.get(
            "computed_governance_commit_ratio", "unknown"
        ),
        "ratio_marker_delta": governance_markers.get("ratio_marker_delta", "unknown"),
        "delta_to_target_ratio_max": delta_to_target_ratio_max,
        "policy_status_within": governance_policy["status_within"],
        "policy_status_over": governance_policy["status_over"],
        "budget_status_marker": governance_markers.get("budget_status_marker", "unknown"),
        "mitigation_issue_marker": governance_markers.get("mitigation_issue_marker", "none"),
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
print(f"governance_release={report['governance_structural_coupling']['release']}")
print(
    "governance_commit_ratio="
    f"{report['governance_structural_coupling']['governance_commit_ratio']}"
)
print(
    "governance_target_ratio_max="
    f"{report['governance_structural_coupling']['target_ratio_max']}"
)
print(
    "governance_structural_coupling_status="
    f"{report['governance_structural_coupling']['status']}"
)
print(
    "governance_delta_to_target_ratio_max="
    f"{report['governance_structural_coupling']['delta_to_target_ratio_max']}"
)
print(
    "governance_budget_status_marker="
    f"{report['governance_structural_coupling']['budget_status_marker']}"
)
print(
    "governance_mitigation_issue_marker="
    f"{report['governance_structural_coupling']['mitigation_issue_marker']}"
)
PY
