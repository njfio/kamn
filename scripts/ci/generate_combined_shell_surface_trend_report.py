#!/usr/bin/env python3

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

GOVERNANCE_RUNTIME_TEST_RATIO_CLASSIFICATION_VERSION = (
    "kamn.ci.governance-runtime-test-ratio-classification.v1"
)
LINE_METRIC_GIT_TIMEOUT_SECONDS = 30
LINE_METRIC_IGNORED_DIRS = {".git", "__pycache__", "node_modules", "target"}


def build_parser(root_dir: Path) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-json", required=True)
    parser.add_argument("--scripts-root", default=str(root_dir / "scripts"))
    parser.add_argument("--rust-root", default=str(root_dir))
    parser.add_argument(
        "--budget-file",
        default=str(root_dir / ".ci" / "script-surface-budget.env"),
    )
    parser.add_argument(
        "--script-baseline-file",
        default=str(root_dir / ".ci" / "script-surface-baseline.env"),
    )
    parser.add_argument(
        "--combined-baseline-file",
        default=str(root_dir / "fixtures" / "ci" / "combined_shell_surface_trend_baseline.json"),
    )
    parser.add_argument(
        "--governance-policy-file",
        default=str(root_dir / "docs" / "review" / "governance-structural-coupling.policy"),
    )
    parser.add_argument(
        "--review-docs-root",
        default=str(root_dir / "docs" / "review"),
    )
    return parser


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
            governance_commit_count / non_merge_commit_count,
            4,
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


def count_lines(paths: list[Path]) -> int:
    return sum(sum(1 for _ in path.open("r", encoding="utf-8", errors="ignore")) for path in paths)


def bounded_file_scan(base_path: Path, suffix: str) -> list[Path]:
    collected: list[Path] = []
    for current_root, dir_names, file_names in os.walk(base_path):
        dir_names[:] = [name for name in dir_names if name not in LINE_METRIC_IGNORED_DIRS]
        for file_name in file_names:
            if file_name.endswith(suffix):
                collected.append(Path(current_root) / file_name)
    return collected


def tracked_files(root_dir: Path, base_path: Path, suffix: str) -> list[Path]:
    try:
        relative_base = base_path.relative_to(root_dir)
    except ValueError:
        return bounded_file_scan(base_path, suffix)
    pathspec = f"*.{suffix.lstrip('.')}" if relative_base == Path(".") else str(relative_base)
    try:
        completed = subprocess.run(
            ["git", "-C", str(root_dir), "ls-files", "--", pathspec],
            check=False,
            capture_output=True,
            text=True,
            timeout=LINE_METRIC_GIT_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise SystemExit("git ls-files timed out while collecting line metrics") from error
    if completed.returncode != 0:
        raise SystemExit(f"git ls-files failed while collecting line metrics: {completed.stderr.strip()}")
    return [root_dir / line for line in completed.stdout.splitlines() if line.endswith(suffix)]


def count_rust_lines(root_dir: Path, rust_root: Path) -> int:
    return count_lines(tracked_files(root_dir, rust_root, ".rs"))


def count_python_lines(root_dir: Path, scripts_root: Path) -> int:
    return count_lines(tracked_files(root_dir, scripts_root, ".py"))


def is_governance_doc_contract_test(path: Path, content: str) -> bool:
    name = path.name
    if name.startswith("review_r"):
        return True
    if name.endswith("_docs.rs") or "docs_contract" in name or "missing_docs_policy" in name:
        return True

    governance_markers = [
        'include_str!("../../../docs/',
        'include_str!("../../docs/',
        'include_str!("../../../README',
        'read_to_string("docs/',
        'read_to_string("tests/docs_contract',
        "docs/review/",
    ]
    return any(marker in content for marker in governance_markers)


def collect_governance_runtime_test_ratio(root_dir: Path) -> dict:
    governance_test_file_count = 0
    runtime_test_file_count = 0
    governance_test_line_total = 0
    runtime_test_line_total = 0

    for path in root_dir.glob("crates/*/tests/**/*.rs"):
        rel_parts = set(path.parts)
        if ".git" in rel_parts or "target" in rel_parts:
            continue
        content = path.read_text(encoding="utf-8", errors="ignore")
        line_total = content.count("\n")
        if content and not content.endswith("\n"):
            line_total += 1
        if is_governance_doc_contract_test(path, content):
            governance_test_file_count += 1
            governance_test_line_total += line_total
        else:
            runtime_test_file_count += 1
            runtime_test_line_total += line_total

    total_test_file_count = governance_test_file_count + runtime_test_file_count
    total_test_line_total = governance_test_line_total + runtime_test_line_total
    governance_test_ratio = (
        round(governance_test_line_total / total_test_line_total, 4)
        if total_test_line_total > 0
        else 0.0
    )
    status = "computed" if total_test_file_count > 0 else "empty"

    return {
        "status": status,
        "classification_version": GOVERNANCE_RUNTIME_TEST_RATIO_CLASSIFICATION_VERSION,
        "test_file_count": total_test_file_count,
        "governance_test_file_count": governance_test_file_count,
        "runtime_test_file_count": runtime_test_file_count,
        "governance_test_line_total": governance_test_line_total,
        "runtime_test_line_total": runtime_test_line_total,
        "total_test_line_total": total_test_line_total,
        "governance_test_ratio": governance_test_ratio,
    }


def main() -> int:
    root_dir = Path(__file__).resolve().parents[2]
    parser = build_parser(root_dir)
    args = parser.parse_args()

    output_path = Path(args.output_json).resolve()
    scripts_root = Path(args.scripts_root).resolve()
    rust_root = Path(args.rust_root).resolve()
    budget_file = Path(args.budget_file).resolve()
    script_baseline_file = Path(args.script_baseline_file).resolve()
    combined_baseline_file = Path(args.combined_baseline_file).resolve()
    governance_policy_file = Path(args.governance_policy_file).resolve()
    review_docs_root = Path(args.review_docs_root).resolve()

    if not budget_file.is_file():
        print(f"budget file not found: {budget_file}", file=sys.stderr)
        return 2
    if not script_baseline_file.is_file():
        print(f"script baseline file not found: {script_baseline_file}", file=sys.stderr)
        return 2
    if not combined_baseline_file.is_file():
        print(f"combined baseline file not found: {combined_baseline_file}", file=sys.stderr)
        return 2
    if not scripts_root.is_dir():
        print(f"scripts root not found: {scripts_root}", file=sys.stderr)
        return 2
    if not rust_root.is_dir():
        print(f"rust root not found: {rust_root}", file=sys.stderr)
        return 2
    if not governance_policy_file.is_file():
        print(
            "governance structural-coupling policy file not found: "
            f"{governance_policy_file}",
            file=sys.stderr,
        )
        return 2
    if not review_docs_root.is_dir():
        print(f"review docs root not found: {review_docs_root}", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory() as tmp_dir:
        script_budget_report = Path(tmp_dir) / "script-surface-budget-report.json"
        check_script = root_dir / "scripts" / "ci" / "check_script_duplication_budget.py"
        process = subprocess.run(
            [
                "python3",
                str(check_script),
                "--scripts-root",
                str(scripts_root),
                "--budget-file",
                str(budget_file),
                "--baseline-file",
                str(script_baseline_file),
                "--output-json",
                str(script_budget_report),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
        script_budget_exit_code = process.returncode
        if not script_budget_report.is_file():
            print(
                f"script budget checker did not emit report: {script_budget_report}",
                file=sys.stderr,
            )
            return 1

        script_budget = json.loads(script_budget_report.read_text(encoding="utf-8"))
        baseline = json.loads(combined_baseline_file.read_text(encoding="utf-8"))
        if baseline.get("schema_version") != "kamn.ci.combined-shell-surface-trend-baseline.v1":
            raise SystemExit("combined baseline schema mismatch")

        metrics = script_budget.get("metrics", {})
        if not isinstance(metrics, dict):
            raise SystemExit("script budget report missing metrics object")

        script_count = int(metrics.get("script_count", 0))
        shell_line_total = int(metrics.get("shell_line_total", 0))

        rust_line_total = count_rust_lines(root_dir, rust_root)
        if rust_line_total <= 0:
            raise SystemExit("rust_line_total must be positive")
        python_line_total = count_python_lines(root_dir, scripts_root)
        if python_line_total <= 0:
            raise SystemExit("python_line_total must be positive")

        shell_to_rust_ratio = round(shell_line_total / rust_line_total, 6)
        baseline_script_count = int(baseline.get("script_count", 0))
        baseline_shell_line_total = int(baseline.get("shell_line_total", 0))
        baseline_rust_line_total = int(baseline.get("rust_line_total", 0))
        baseline_python_line_total = int(baseline.get("python_line_total", 0))
        baseline_shell_to_rust_ratio = float(baseline.get("shell_to_rust_ratio", 0.0))

        delta_script_count = script_count - baseline_script_count
        delta_shell_line_total = shell_line_total - baseline_shell_line_total
        delta_rust_line_total = rust_line_total - baseline_rust_line_total
        delta_python_line_total = python_line_total - baseline_python_line_total
        delta_shell_to_rust_ratio = round(shell_to_rust_ratio - baseline_shell_to_rust_ratio, 6)

        governance_policy = parse_policy(governance_policy_file)
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
                and str(governance_markers.get("mitigation_issue_marker", "")).strip()
                not in ("", "none")
            ):
                governance_status = "reduction_contract_active"
            else:
                governance_status = "over_target_unmitigated"
        governance_runtime_test_ratio = collect_governance_runtime_test_ratio(root_dir)

        report = {
            "schema_version": "kamn.ci.combined-shell-surface-trend-report.v1",
            "status": "generated",
            "generated_at_utc": dt.datetime.now(tz=dt.timezone.utc).isoformat(),
            "current": {
                "script_count": script_count,
                "shell_line_total": shell_line_total,
                "rust_line_total": rust_line_total,
                "python_line_total": python_line_total,
                "shell_to_rust_ratio": shell_to_rust_ratio,
            },
            "baseline": {
                "script_count": baseline_script_count,
                "shell_line_total": baseline_shell_line_total,
                "rust_line_total": baseline_rust_line_total,
                "python_line_total": baseline_python_line_total,
                "shell_to_rust_ratio": baseline_shell_to_rust_ratio,
            },
            "deltas": {
                "script_count": delta_script_count,
                "shell_line_total": delta_shell_line_total,
                "rust_line_total": delta_rust_line_total,
                "python_line_total": delta_python_line_total,
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
                    "target_ratio_max_next_release_marker",
                    "unknown",
                ),
                "non_merge_commit_count": governance_markers.get(
                    "non_merge_commit_count",
                    "unknown",
                ),
                "governance_commit_count": governance_markers.get(
                    "governance_commit_count",
                    "unknown",
                ),
                "governance_commit_ratio": governance_markers.get(
                    "governance_commit_ratio",
                    "unknown",
                ),
                "computed_governance_commit_ratio": governance_markers.get(
                    "computed_governance_commit_ratio",
                    "unknown",
                ),
                "ratio_marker_delta": governance_markers.get("ratio_marker_delta", "unknown"),
                "delta_to_target_ratio_max": delta_to_target_ratio_max,
                "policy_status_within": governance_policy["status_within"],
                "policy_status_over": governance_policy["status_over"],
                "budget_status_marker": governance_markers.get("budget_status_marker", "unknown"),
                "mitigation_issue_marker": governance_markers.get(
                    "mitigation_issue_marker",
                    "none",
                ),
            },
            "governance_runtime_test_ratio": governance_runtime_test_ratio,
        }

        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

        print("status=generated")
        print(f"script_count={script_count}")
        print(f"shell_line_total={shell_line_total}")
        print(f"rust_line_total={rust_line_total}")
        print(f"python_line_total={python_line_total}")
        print(f"shell_to_rust_ratio={shell_to_rust_ratio}")
        print(f"delta_script_count={delta_script_count}")
        print(f"delta_shell_line_total={delta_shell_line_total}")
        print(f"delta_rust_line_total={delta_rust_line_total}")
        print(f"delta_python_line_total={delta_python_line_total}")
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
        print(
            "governance_runtime_test_ratio="
            f"{report['governance_runtime_test_ratio']['governance_test_ratio']}"
        )
        print(
            "governance_test_line_total="
            f"{report['governance_runtime_test_ratio']['governance_test_line_total']}"
        )
        print(
            "runtime_test_line_total="
            f"{report['governance_runtime_test_ratio']['runtime_test_line_total']}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
