#!/usr/bin/env python3
"""Fail closed when panic-style or unsafe fallback patterns appear in production Rust code."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
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

DEFAULT_RUNTIME_ROOTS = (
    "crates/kamn-agent-lib/src",
    "crates/kamn-core/src",
    "crates/kamn-node/src",
    "crates/kamn-cli/src",
    "crates/kamn-mcp-server/src",
    "crates/kamn-sdk/src",
    "crates/kamn-kolme/src",
    "crates/kamn-e2e-harness/src",
)
TEST_ONLY_PATH_MARKERS = (
    "/main_tests/",
    "/runtime_tests/",
    "/cli_tests/",
    "/cli_scripted_tests/",
    "/mcp_agent_tests/",
    "/tests/",
    "/test_utils/",
)


def is_excluded(path: Path) -> bool:
    path_str = path.as_posix()
    file_name = path.name
    return (
        any(marker in path_str for marker in TEST_ONLY_PATH_MARKERS)
        or ("/src/" in path_str and file_name == "tests.rs")
        or file_name == "test_support.rs"
        or file_name.endswith("_tests.rs")
        or "_tests_" in file_name
        or file_name.startswith("runtime_tests")
        or file_name.startswith("cli_tests")
        or file_name == "main_tests.rs"
    )


def is_test_cfg_attribute(line: str) -> bool:
    stripped = line.strip()
    return (
        stripped.startswith("#[cfg(")
        and "test" in stripped
        and "not(test)" not in stripped
    )


@dataclass
class BraceScanState:
    in_block_comment: bool = False
    in_string: bool = False
    escape_next: bool = False
    raw_string_hashes: int | None = None


def match_raw_string_start(line: str, index: int) -> tuple[int, int] | None:
    if line[index] != "r":
        return None
    cursor = index + 1
    hashes = 0
    while cursor < len(line) and line[cursor] == "#":
        hashes += 1
        cursor += 1
    if cursor < len(line) and line[cursor] == '"':
        return hashes, cursor + 1
    return None


def match_raw_string_end(line: str, index: int, hashes: int) -> int | None:
    if line[index] != '"':
        return None
    cursor = index + 1
    for _ in range(hashes):
        if cursor >= len(line) or line[cursor] != "#":
            return None
        cursor += 1
    return cursor


def count_code_braces(line: str, state: BraceScanState) -> tuple[int, int]:
    open_count = 0
    close_count = 0
    index = 0

    while index < len(line):
        if state.raw_string_hashes is not None:
            if line[index] == '"':
                raw_end = match_raw_string_end(line, index, state.raw_string_hashes)
                if raw_end is not None:
                    state.raw_string_hashes = None
                    index = raw_end
                    continue
            index += 1
            continue

        if state.in_block_comment:
            if line.startswith("*/", index):
                state.in_block_comment = False
                index += 2
            else:
                index += 1
            continue

        if state.in_string:
            if state.escape_next:
                state.escape_next = False
                index += 1
                continue
            if line[index] == "\\":
                state.escape_next = True
                index += 1
                continue
            if line[index] == '"':
                state.in_string = False
            index += 1
            continue

        if line.startswith("//", index):
            break

        if line.startswith("/*", index):
            state.in_block_comment = True
            index += 2
            continue

        raw_start = match_raw_string_start(line, index)
        if raw_start is not None:
            state.raw_string_hashes, index = raw_start
            continue

        if line[index] == '"':
            state.in_string = True
            state.escape_next = False
            index += 1
            continue

        if line[index] == "{":
            open_count += 1
        elif line[index] == "}":
            close_count += 1
        index += 1

    return open_count, close_count


def skip_cfg_test_item(lines: list[str], index: int) -> int:
    while index < len(lines) and lines[index].strip() == "":
        index += 1

    while index < len(lines) and lines[index].lstrip().startswith("#["):
        index += 1

    if index >= len(lines):
        return index

    brace_depth = 0
    saw_open_brace = False
    scan_state = BraceScanState()
    while index < len(lines):
        line = lines[index]
        open_count, close_count = count_code_braces(line, scan_state)
        if open_count > 0:
            saw_open_brace = True
        brace_depth += open_count - close_count
        index += 1

        if saw_open_brace:
            if brace_depth <= 0:
                return index
            continue

        if line.rstrip().endswith(";"):
            return index

    return index


def iter_production_lines(lines: list[str]):
    index = 0
    while index < len(lines):
        if is_test_cfg_attribute(lines[index]):
            index = skip_cfg_test_item(lines, index + 1)
            continue
        yield index + 1, lines[index]
        index += 1


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
        for line_no, line in iter_production_lines(lines):
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
        help=(
            "Rust source root to scan (repeatable). Default: "
            + ",".join(DEFAULT_RUNTIME_ROOTS)
        ),
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for deterministic check report JSON.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    roots = args.root or list(DEFAULT_RUNTIME_ROOTS)

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
