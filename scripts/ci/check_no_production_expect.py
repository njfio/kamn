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


def is_excluded(path: Path) -> bool:
    path_str = path.as_posix()
    name = path.name
    return (
        "/main_tests/" in path_str
        or "/runtime_tests/" in path_str
        or "/cli_tests/" in path_str
        or "/test_utils/" in path_str
        or "/tests/" in path_str
        or name == "main_tests.rs"
        or "_tests" in name
    )


def extract_cfg_expression(line: str) -> str | None:
    stripped = line.strip()
    if not (stripped.startswith("#[cfg(") and stripped.endswith(")]")):
        return None
    return stripped[len("#[cfg(") : -2]


@dataclass(frozen=True)
class TruthSet:
    possible_true: bool
    possible_false: bool


def split_top_level_cfg_args(expr: str) -> list[str]:
    args: list[str] = []
    start = 0
    depth = 0
    in_string = False
    escape_next = False
    for index, ch in enumerate(expr):
        if in_string:
            if escape_next:
                escape_next = False
                continue
            if ch == "\\":
                escape_next = True
                continue
            if ch == '"':
                in_string = False
            continue
        if ch == '"':
            in_string = True
            continue
        if ch == "(":
            depth += 1
            continue
        if ch == ")":
            depth -= 1
            continue
        if ch == "," and depth == 0:
            value = expr[start:index].strip()
            if value:
                args.append(value)
            start = index + 1
    tail = expr[start:].strip()
    if tail:
        args.append(tail)
    return args


def cfg_strip_outer_call(expr: str, name: str) -> str | None:
    expr = expr.strip()
    prefix = f"{name}("
    if not (expr.startswith(prefix) and expr.endswith(")")):
        return None

    depth = 0
    in_string = False
    escape_next = False
    for index, ch in enumerate(expr):
        if in_string:
            if escape_next:
                escape_next = False
                continue
            if ch == "\\":
                escape_next = True
                continue
            if ch == '"':
                in_string = False
            continue
        if ch == '"':
            in_string = True
            continue
        if ch == "(":
            depth += 1
            continue
        if ch == ")":
            depth -= 1
            if depth == 0 and index != len(expr) - 1:
                return None
    if depth != 0:
        return None
    return expr[len(prefix) : -1]


def cfg_truth_set(expr: str, test_enabled: bool) -> TruthSet:
    expr = expr.strip()
    if expr == "":
        return TruthSet(True, True)

    all_inner = cfg_strip_outer_call(expr, "all")
    if all_inner is not None:
        args = split_top_level_cfg_args(all_inner)
        if not args:
            return TruthSet(True, True)
        possible_true = True
        possible_false = False
        for arg in args:
            child = cfg_truth_set(arg, test_enabled)
            possible_true = possible_true and child.possible_true
            possible_false = possible_false or child.possible_false
        return TruthSet(possible_true, possible_false)

    any_inner = cfg_strip_outer_call(expr, "any")
    if any_inner is not None:
        args = split_top_level_cfg_args(any_inner)
        if not args:
            return TruthSet(True, True)
        possible_true = False
        possible_false = True
        for arg in args:
            child = cfg_truth_set(arg, test_enabled)
            possible_true = possible_true or child.possible_true
            possible_false = possible_false and child.possible_false
        return TruthSet(possible_true, possible_false)

    not_inner = cfg_strip_outer_call(expr, "not")
    if not_inner is not None:
        child = cfg_truth_set(not_inner, test_enabled)
        return TruthSet(child.possible_false, child.possible_true)

    if expr == "test":
        if test_enabled:
            return TruthSet(True, False)
        return TruthSet(False, True)

    # Non-test predicates are treated as environment/feature dependent.
    return TruthSet(True, True)


def cfg_expression_requires_test(expr: str) -> bool:
    return not cfg_truth_set(expr, test_enabled=False).possible_true


def is_test_cfg_attribute(line: str) -> bool:
    cfg_expr = extract_cfg_expression(line)
    if cfg_expr is None:
        return False
    return cfg_expression_requires_test(cfg_expr)


@dataclass
class BraceScanState:
    block_comment_depth: int = 0
    string_delimiter: str | None = None
    raw_string_hashes: int | None = None
    escape_next: bool = False


def is_ident_byte(ch: str) -> bool:
    return ch.isalnum() or ch == "_"


def parse_raw_string_start(line: str, index: int) -> tuple[int, int] | None:
    if index >= len(line):
        return None

    prefix_len = 0
    if line.startswith("br", index):
        prefix_len = 2
    elif line[index] == "r":
        prefix_len = 1
    else:
        return None

    if index > 0 and is_ident_byte(line[index - 1]):
        return None

    cursor = index + prefix_len
    hash_count = 0
    while cursor < len(line) and line[cursor] == "#":
        hash_count += 1
        cursor += 1

    if cursor < len(line) and line[cursor] == '"':
        return hash_count, cursor + 1
    return None


def starts_char_literal(line: str, index: int) -> bool:
    if index >= len(line) or line[index] != "'":
        return False
    if index + 1 >= len(line):
        return False
    next_ch = line[index + 1]
    return not (next_ch.isalpha() or next_ch == "_")


def scan_line_brace_counts(line: str, state: BraceScanState) -> tuple[int, int]:
    open_count = 0
    close_count = 0
    index = 0
    line_len = len(line)

    while index < line_len:
        ch = line[index]

        if state.raw_string_hashes is not None:
            if ch == '"':
                suffix = "#" * state.raw_string_hashes
                if line.startswith(suffix, index + 1):
                    index += 1 + state.raw_string_hashes
                    state.raw_string_hashes = None
                    continue
            index += 1
            continue

        if state.string_delimiter is not None:
            if state.escape_next:
                state.escape_next = False
                index += 1
                continue
            if ch == "\\":
                state.escape_next = True
                index += 1
                continue
            if ch == state.string_delimiter:
                state.string_delimiter = None
            index += 1
            continue

        if state.block_comment_depth > 0:
            if line.startswith("/*", index):
                state.block_comment_depth += 1
                index += 2
                continue
            if line.startswith("*/", index):
                state.block_comment_depth -= 1
                index += 2
                continue
            index += 1
            continue

        if line.startswith("//", index):
            break
        if line.startswith("/*", index):
            state.block_comment_depth += 1
            index += 2
            continue

        raw_string = parse_raw_string_start(line, index)
        if raw_string is not None:
            state.raw_string_hashes, index = raw_string
            state.escape_next = False
            continue

        if line.startswith('b"', index) and (index == 0 or not is_ident_byte(line[index - 1])):
            state.string_delimiter = '"'
            state.escape_next = False
            index += 2
            continue
        if line.startswith("b'", index) and (index == 0 or not is_ident_byte(line[index - 1])):
            state.string_delimiter = "'"
            state.escape_next = False
            index += 2
            continue
        if ch == '"':
            state.string_delimiter = ch
            state.escape_next = False
            index += 1
            continue
        if ch == "'" and starts_char_literal(line, index):
            state.string_delimiter = ch
            state.escape_next = False
            index += 1
            continue

        if ch == "{":
            open_count += 1
        elif ch == "}":
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

    scan_state = BraceScanState()
    brace_depth = 0
    saw_open_brace = False
    while index < len(lines):
        line = lines[index]
        open_count, close_count = scan_line_brace_counts(line, scan_state)
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
