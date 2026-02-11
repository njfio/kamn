#!/usr/bin/env python3
"""Generate and validate kamn-core missing-docs throughput reports."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "kamn.ci.kamn-core-missing-docs-throughput-report.v1"
CRATE_NAME = "kamn-core"
TARGET_MODULES_PER_100_COMMITS = 5
REASON_TARGET_MET = "throughput_target_met"
REASON_TARGET_UNDER = "throughput_target_under_target"


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def parse_fixture_lines(path: Path) -> list[str]:
    lines: list[str] = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        lines.append(line)
    return sorted(lines)


def parse_public_modules(core_lib: Path) -> list[str]:
    modules: list[str] = []
    pattern = re.compile(r"^\s*pub mod ([a-zA-Z0-9_]+);")
    for raw_line in core_lib.read_text(encoding="utf-8").splitlines():
        match = pattern.match(raw_line)
        if match:
            modules.append(match.group(1))
    return sorted(modules)


def git_commit_count(root: Path) -> int:
    completed = subprocess.run(
        ["git", "rev-list", "--count", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    count = int(completed.stdout.strip())
    if count <= 0:
        raise ValueError("commit_count must be positive")
    return count


def generate_report(args: argparse.Namespace) -> int:
    root = repo_root()
    core_lib = Path(args.core_lib)
    allowlist_path = Path(args.allowlist)
    graduated_modules_path = Path(args.graduated_modules)
    output_path = Path(args.output_json)

    allowlisted_modules = parse_fixture_lines(allowlist_path)
    graduated_modules = parse_fixture_lines(graduated_modules_path)
    public_modules = parse_public_modules(core_lib)

    commit_count = args.commit_count
    if commit_count is None:
        commit_count = git_commit_count(root)
    if commit_count <= 0:
        raise ValueError("commit_count must be positive")

    observed = (len(graduated_modules) * 100.0) / float(commit_count)
    observed = round(observed, 4)
    target_met = observed >= float(args.target_modules_per_100_commits)
    reason_key = REASON_TARGET_MET if target_met else REASON_TARGET_UNDER

    report: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "crate": CRATE_NAME,
        "commit_count": commit_count,
        "target_modules_per_100_commits": int(args.target_modules_per_100_commits),
        "observed_modules_per_100_commits": observed,
        "graduated_module_count": len(graduated_modules),
        "allowlisted_module_count": len(allowlisted_modules),
        "total_public_module_count": len(public_modules),
        "graduated_modules": graduated_modules,
        "target_met": target_met,
        "reason_key": reason_key,
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"generated missing-docs throughput report: {output_path}")
    return 0


def require_key(payload: dict[str, Any], key: str, expected_type: type) -> Any:
    if key not in payload:
        raise ValueError(f"missing key: {key}")
    value = payload[key]
    if not isinstance(value, expected_type):
        raise ValueError(f"invalid type for {key}: expected {expected_type.__name__}")
    return value


def check_report(args: argparse.Namespace) -> int:
    report_path = Path(args.report_file)
    payload = json.loads(report_path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("report payload must be a JSON object")

    schema_version = require_key(payload, "schema_version", str)
    if schema_version != SCHEMA_VERSION:
        raise ValueError(f"schema_version must be {SCHEMA_VERSION}")

    crate = require_key(payload, "crate", str)
    if crate != CRATE_NAME:
        raise ValueError(f"crate must be {CRATE_NAME}")

    commit_count = require_key(payload, "commit_count", int)
    if commit_count <= 0:
        raise ValueError("commit_count must be positive")

    target = require_key(payload, "target_modules_per_100_commits", int)
    if target <= 0:
        raise ValueError("target_modules_per_100_commits must be positive")

    observed = require_key(payload, "observed_modules_per_100_commits", (int, float))
    if observed < 0:
        raise ValueError("observed_modules_per_100_commits must be non-negative")

    graduated_module_count = require_key(payload, "graduated_module_count", int)
    allowlisted_module_count = require_key(payload, "allowlisted_module_count", int)
    total_public_module_count = require_key(payload, "total_public_module_count", int)
    if graduated_module_count < 0 or allowlisted_module_count < 0 or total_public_module_count <= 0:
        raise ValueError("module counts must be non-negative and total_public_module_count must be positive")

    graduated_modules = require_key(payload, "graduated_modules", list)
    if sorted(graduated_modules) != graduated_modules:
        raise ValueError("graduated_modules must be sorted")
    if len(graduated_modules) != graduated_module_count:
        raise ValueError("graduated_module_count must match graduated_modules length")

    target_met = require_key(payload, "target_met", bool)
    expected_target_met = float(observed) >= float(target)
    if target_met != expected_target_met:
        raise ValueError("target_met must match observed_modules_per_100_commits >= target_modules_per_100_commits")

    reason_key = require_key(payload, "reason_key", str)
    expected_reason_key = REASON_TARGET_MET if target_met else REASON_TARGET_UNDER
    if reason_key != expected_reason_key:
        raise ValueError(f"reason_key must be {expected_reason_key}")

    print("missing-docs throughput report policy passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    root = repo_root()
    parser = argparse.ArgumentParser(
        description="Generate and validate kamn-core missing-docs throughput reports."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument(
        "--core-lib",
        default=str(root / "crates/kamn-core/src/lib.rs"),
    )
    generate.add_argument(
        "--allowlist",
        default=str(root / "fixtures/ci/kamn_core_missing_docs_allowlist.txt"),
    )
    generate.add_argument(
        "--graduated-modules",
        default=str(root / "fixtures/ci/kamn_core_missing_docs_graduated_modules.txt"),
    )
    generate.add_argument(
        "--target-modules-per-100-commits",
        type=int,
        default=TARGET_MODULES_PER_100_COMMITS,
    )
    generate.add_argument(
        "--commit-count",
        type=int,
        default=None,
    )
    generate.add_argument("--output-json", required=True)

    check = subparsers.add_parser("check")
    check.add_argument("--report-file", required=True)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        if args.command == "generate":
            return generate_report(args)
        if args.command == "check":
            return check_report(args)
    except ValueError as error:
        print(f"missing-docs throughput report policy failed: {error}", file=sys.stderr)
        return 1
    raise AssertionError(f"unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
