#!/usr/bin/env python3
"""Fail if `.expect(` appears in production Rust code before `#[cfg(test)]` blocks."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def is_excluded(path: Path) -> bool:
    path_str = path.as_posix()
    return (
        "/main_tests/" in path_str
        or path.name.endswith("_tests.rs")
        or path.name == "main_tests.rs"
    )


def has_unsafe_env_fallback_default(line: str) -> bool:
    compact = line.replace(" ", "")
    has_env_var = "std::env::var(" in compact or "env::var(" in compact
    has_default_fallback = ".unwrap_or(" in compact or ".unwrap_or_else(" in compact
    return has_env_var and has_default_fallback


def find_violations(root: Path) -> list[dict[str, object]]:
    violations: list[dict[str, object]] = []
    for rust_file in sorted(root.rglob("*.rs")):
        if is_excluded(rust_file):
            continue
        lines = rust_file.read_text(encoding="utf-8").splitlines()
        cutoff = len(lines)
        for line_no, line in enumerate(lines, start=1):
            if line.strip().startswith("#[cfg(test)]"):
                cutoff = line_no - 1
                break
        for line_no, line in enumerate(lines[:cutoff], start=1):
            if (
                ".expect(" in line
                or "panic!(" in line
                or "unreachable!(" in line
                or has_unsafe_env_fallback_default(line)
            ):
                violations.append(
                    {
                        "file": rust_file.as_posix(),
                        "line": line_no,
                        "snippet": line.strip(),
                    }
                )
    return violations


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        action="append",
        default=[],
        help="Rust source root to scan (repeatable). Default: crates/kamn-node/src",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for deterministic check report JSON.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    roots = args.root or ["crates/kamn-node/src"]

    report: dict[str, object] = {
        "schema_version": "kamn.ci.no-production-expect-report.v1",
        "roots": roots,
        "violations": [],
        "violation_count": 0,
        "status": "pass",
    }

    all_violations: list[dict[str, object]] = []
    for root in roots:
        root_path = Path(root)
        if not root_path.exists():
            report["status"] = "fail"
            all_violations.append(
                {
                    "file": root,
                    "line": 0,
                    "snippet": "scan root does not exist",
                }
            )
            continue
        all_violations.extend(find_violations(root_path))

    report["violations"] = all_violations
    report["violation_count"] = len(all_violations)

    if all_violations:
        report["status"] = "fail"

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if report["status"] == "pass":
        print("status=ok")
        print("violation_count=0")
        return 0

    print("status=fail")
    print(f"violation_count={report['violation_count']}")
    for violation in all_violations:
        print(
            "violation="
            f"{violation['file']}:{violation['line']}:{violation['snippet']}"
        )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
