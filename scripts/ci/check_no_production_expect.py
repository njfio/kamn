#!/usr/bin/env python3
"""Fail closed when panic-style or unsafe fallback patterns appear in production Rust code."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


REASON_TAXONOMY_VERSION = "kamn.ci.production-panic-replacement-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "scan_root_not_found,"
    "production_expect_reachable,"
    "production_panic_macro_reachable,"
    "production_unreachable_macro_reachable,"
    "production_unsafe_env_fallback_default"
)
REASON_CODES_ORDER = tuple(REASON_CODES_CSV.split(","))
RUNTIME_EVIDENCE_OUTPUTS_CSV = (
    "runtime_panic_replacement_evidence_status,"
    "runtime_panic_replacement_evidence_violation_count,"
    "runtime_panic_replacement_evidence_files_csv"
)
PANIC_REASON_CODES = frozenset(
    {
        "production_expect_reachable",
        "production_panic_macro_reachable",
        "production_unreachable_macro_reachable",
    }
)


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


def reason_code_for_line(line: str) -> str | None:
    if ".expect(" in line:
        return "production_expect_reachable"
    if "panic!(" in line:
        return "production_panic_macro_reachable"
    if "unreachable!(" in line:
        return "production_unreachable_macro_reachable"
    if has_unsafe_env_fallback_default(line):
        return "production_unsafe_env_fallback_default"
    return None


def normalize_reason_codes(violations: list[dict[str, object]]) -> list[str]:
    observed = {
        str(violation.get("reason_code", ""))
        for violation in violations
        if str(violation.get("reason_code", "")) != ""
    }
    return [reason_code for reason_code in REASON_CODES_ORDER if reason_code in observed]


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def classify_reason_class(reason_codes: list[str]) -> str:
    if not reason_codes:
        return "stable"
    has_configuration = "scan_root_not_found" in reason_codes
    has_panic_reachability = any(reason_code in PANIC_REASON_CODES for reason_code in reason_codes)
    has_unsafe_fallback = "production_unsafe_env_fallback_default" in reason_codes
    if has_configuration and not has_panic_reachability and not has_unsafe_fallback:
        return "configuration"
    if has_panic_reachability and has_unsafe_fallback:
        return "mixed"
    if has_panic_reachability:
        return "panic_reachability"
    if has_unsafe_fallback:
        return "unsafe_fallback"
    return "configuration"


def runtime_evidence_files_csv(violations: list[dict[str, object]]) -> str:
    if not violations:
        return "none"
    files = sorted(
        {
            str(violation.get("file", ""))
            for violation in violations
            if str(violation.get("file", "")) != ""
        }
    )
    return ",".join(files) if files else "none"


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
            reason_code = reason_code_for_line(line)
            if reason_code is not None:
                violations.append(
                    {
                        "file": rust_file.as_posix(),
                        "line": line_no,
                        "reason_code": reason_code,
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
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": [],
        "reason_codes_value": "none",
        "reason_class": "stable",
        "runtime_panic_replacement_evidence_status": "verified",
        "runtime_panic_replacement_evidence_violation_count": 0,
        "runtime_panic_replacement_evidence_files_csv": "none",
        "runtime_panic_replacement_evidence_outputs_csv": RUNTIME_EVIDENCE_OUTPUTS_CSV,
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
                    "reason_code": "scan_root_not_found",
                    "snippet": "scan root does not exist",
                }
            )
            continue
        all_violations.extend(find_violations(root_path))

    normalized_reason_codes = normalize_reason_codes(all_violations)
    normalized_reason_codes_value = reason_codes_value(normalized_reason_codes)
    normalized_reason_class = classify_reason_class(normalized_reason_codes)
    evidence_files_csv = runtime_evidence_files_csv(all_violations)
    evidence_status = "verified" if not all_violations else "violation"

    report["violations"] = all_violations
    report["violation_count"] = len(all_violations)
    report["reason_codes"] = normalized_reason_codes
    report["reason_codes_value"] = normalized_reason_codes_value
    report["reason_class"] = normalized_reason_class
    report["runtime_panic_replacement_evidence_status"] = evidence_status
    report["runtime_panic_replacement_evidence_violation_count"] = len(all_violations)
    report["runtime_panic_replacement_evidence_files_csv"] = evidence_files_csv

    if all_violations:
        report["status"] = "fail"

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("status=ok" if report["status"] == "pass" else "status=fail")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes_value={normalized_reason_codes_value}")
    print(f"reason_class={normalized_reason_class}")
    print(f"runtime_panic_replacement_evidence_status={evidence_status}")
    print(f"runtime_panic_replacement_evidence_violation_count={len(all_violations)}")
    print(f"runtime_panic_replacement_evidence_files_csv={evidence_files_csv}")
    print(f"runtime_panic_replacement_evidence_outputs_csv={RUNTIME_EVIDENCE_OUTPUTS_CSV}")
    if report["status"] == "pass":
        print("violation_count=0")
        return 0

    print(f"violation_count={report['violation_count']}")
    for violation in all_violations:
        print(
            "violation="
            f"{violation['file']}:{violation['line']}:{violation['reason_code']}:{violation['snippet']}"
        )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
