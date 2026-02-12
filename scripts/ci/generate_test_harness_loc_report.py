#!/usr/bin/env python3
"""Generate deterministic shell test-harness LOC metrics."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys


SCHEMA_VERSION = "kamn.ci.test-harness-loc-report.v1"


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate deterministic shell test-harness LOC metrics."
    )
    parser.add_argument(
        "--scripts-root",
        default="scripts",
        help="Root directory to scan for shell harness scripts (default: scripts).",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output report path.",
    )
    return parser.parse_args(argv)


def count_lines(path: Path) -> int:
    return sum(1 for _ in path.open("r", encoding="utf-8", errors="ignore"))


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    scripts_root = Path(args.scripts_root)
    if not scripts_root.is_dir():
        print("status=fail")
        print(f"error=scripts root not found: {scripts_root}")
        return 1

    harness_scripts = sorted(
        path
        for path in scripts_root.rglob("test_*.sh")
        if path.is_file()
    )
    harness_script_count = len(harness_scripts)
    harness_shell_line_total = 0
    domain_metrics: dict[str, dict[str, int]] = {}

    harness_script_paths: list[str] = []
    for path in harness_scripts:
        line_total = count_lines(path)
        harness_shell_line_total += line_total
        relative = path.relative_to(scripts_root)
        harness_script_paths.append(str(relative))
        domain = relative.parts[0] if len(relative.parts) > 1 else "_root"
        domain_summary = domain_metrics.setdefault(
            domain, {"script_count": 0, "shell_line_total": 0}
        )
        domain_summary["script_count"] += 1
        domain_summary["shell_line_total"] += line_total

    ordered_domains = {
        key: domain_metrics[key]
        for key in sorted(domain_metrics.keys())
    }

    report = {
        "schema_version": SCHEMA_VERSION,
        "scripts_root": str(scripts_root.resolve()),
        "harness_script_count": harness_script_count,
        "harness_shell_line_total": harness_shell_line_total,
        "domains": ordered_domains,
        "harness_scripts": harness_script_paths,
    }

    report_path: Path | None = None
    if args.output_json:
        report_path = Path(args.output_json).resolve()
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(
            json.dumps(report, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print("status=ok")
    print(f"harness_script_count={harness_script_count}")
    print(f"harness_shell_line_total={harness_shell_line_total}")
    if report_path is not None:
        print(f"report_file={report_path}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
