#!/usr/bin/env python3
"""Generate and validate deterministic ignored-test inventory baselines."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

INVENTORY_SCHEMA_VERSION = "kamn.ci.ignored-test-inventory.v1"
DRIFT_REPORT_SCHEMA_VERSION = "kamn.ci.ignored-test-inventory-drift-report.v1"
DEFAULT_SCAN_ROOT = "crates"
DEFAULT_BASELINE_FILE = "fixtures/ci/ignored_test_inventory_baseline.json"

IGNORE_ATTRIBUTE_PATTERN = re.compile(r"^\s*#\s*\[\s*ignore(?:\s*=.*)?\s*\]")
FUNCTION_PATTERN = re.compile(r"^\s*(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")


class CheckerError(RuntimeError):
    """Raised when checker configuration or input is invalid."""


def fail(message: str) -> None:
    raise CheckerError(message)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate/check fail-closed ignored-test inventory drift contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser(
        "generate", help="Generate deterministic ignored-test inventory baseline."
    )
    generate_parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root used to resolve scan roots and output paths.",
    )
    generate_parser.add_argument(
        "--scan-root",
        action="append",
        default=[],
        help=(
            "Relative scan root for Rust files. Can be supplied multiple times. "
            f"Defaults to {DEFAULT_SCAN_ROOT}."
        ),
    )
    generate_parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write generated inventory JSON.",
    )

    check_parser = subparsers.add_parser(
        "check", help="Check ignored-test inventory against a baseline fixture."
    )
    check_parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root used to resolve scan roots and baseline path.",
    )
    check_parser.add_argument(
        "--scan-root",
        action="append",
        default=[],
        help=(
            "Relative scan root for Rust files. Can be supplied multiple times. "
            f"Defaults to {DEFAULT_SCAN_ROOT}."
        ),
    )
    check_parser.add_argument(
        "--baseline-file",
        default=DEFAULT_BASELINE_FILE,
        help="Path to ignored-test inventory baseline fixture.",
    )
    check_parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write drift-check report JSON.",
    )

    return parser.parse_args(argv)


def resolve_path(*, repo_root: Path, value: str, label: str) -> Path:
    path = Path(value)
    if not path.is_absolute():
        path = (repo_root / path).resolve()
    if not path.exists():
        fail(f"{label} not found: {path}")
    return path


def to_repo_relative(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def read_rust_source(path: Path) -> list[str]:
    try:
        return path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        fail(f"failed to read Rust source {path}: {exc}")


def collect_ignored_tests(*, repo_root: Path, scan_roots: list[str]) -> tuple[list[dict[str, str]], list[str]]:
    roots = scan_roots or [DEFAULT_SCAN_ROOT]
    ignored_tests: list[dict[str, str]] = []
    unresolved_markers: list[str] = []

    for root_value in roots:
        root_path = resolve_path(repo_root=repo_root, value=root_value, label="scan root")
        if not root_path.is_dir():
            fail(f"scan root is not a directory: {root_path}")

        for rust_file in sorted(path for path in root_path.rglob("*.rs") if path.is_file()):
            repo_relative = to_repo_relative(rust_file, repo_root)
            pending_ignore_line: int | None = None

            for line_number, line in enumerate(read_rust_source(rust_file), start=1):
                if IGNORE_ATTRIBUTE_PATTERN.match(line):
                    if pending_ignore_line is not None:
                        unresolved_markers.append(
                            f"{repo_relative}:{pending_ignore_line}:ignore_attribute_not_bound_to_function"
                        )
                    pending_ignore_line = line_number
                    continue

                if pending_ignore_line is None:
                    continue

                stripped = line.strip()
                if (
                    stripped == ""
                    or stripped.startswith("//")
                    or stripped.startswith("/*")
                    or stripped.startswith("*")
                    or stripped.startswith("*/")
                    or stripped.startswith("#[")
                ):
                    continue

                function_match = FUNCTION_PATTERN.match(line)
                if function_match:
                    ignored_tests.append(
                        {
                            "source_file": repo_relative,
                            "test_name": function_match.group(1),
                        }
                    )
                    pending_ignore_line = None

            if pending_ignore_line is not None:
                unresolved_markers.append(
                    f"{repo_relative}:{pending_ignore_line}:ignore_attribute_without_function"
                )

    ignored_tests.sort(key=lambda item: (item["source_file"], item["test_name"]))
    return ignored_tests, sorted(unresolved_markers)


def validate_inventory_payload(payload: dict[str, Any], *, label: str) -> list[dict[str, str]]:
    if payload.get("schema_version") != INVENTORY_SCHEMA_VERSION:
        fail(f"{label} schema_version must be {INVENTORY_SCHEMA_VERSION}")

    ignored_test_count = payload.get("ignored_test_count")
    if not isinstance(ignored_test_count, int) or ignored_test_count < 0:
        fail(f"{label} ignored_test_count must be a non-negative integer")

    ignored_tests = payload.get("ignored_tests")
    if not isinstance(ignored_tests, list):
        fail(f"{label} ignored_tests must be an array")

    normalized: list[dict[str, str]] = []
    seen_keys: set[tuple[str, str]] = set()
    for index, entry in enumerate(ignored_tests):
        if not isinstance(entry, dict):
            fail(f"{label} ignored_tests[{index}] must be an object")
        source_file = entry.get("source_file")
        test_name = entry.get("test_name")
        if not isinstance(source_file, str) or not source_file.strip():
            fail(f"{label} ignored_tests[{index}].source_file must be a non-empty string")
        if not isinstance(test_name, str) or not test_name.strip():
            fail(f"{label} ignored_tests[{index}].test_name must be a non-empty string")
        key = (source_file.strip(), test_name.strip())
        if key in seen_keys:
            fail(
                f"{label} ignored_tests must be unique; duplicate entry detected: {key[0]}::{key[1]}"
            )
        seen_keys.add(key)
        normalized.append({"source_file": key[0], "test_name": key[1]})

    normalized.sort(key=lambda item: (item["source_file"], item["test_name"]))
    if len(normalized) != ignored_test_count:
        fail(
            f"{label} ignored_test_count mismatch: expected {len(normalized)} from entries, got {ignored_test_count}"
        )

    return normalized


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def generate_inventory(
    *,
    repo_root: Path,
    scan_roots: list[str],
) -> tuple[dict[str, Any], list[str]]:
    ignored_tests, unresolved_markers = collect_ignored_tests(
        repo_root=repo_root,
        scan_roots=scan_roots,
    )
    inventory = {
        "schema_version": INVENTORY_SCHEMA_VERSION,
        "scan_roots": scan_roots or [DEFAULT_SCAN_ROOT],
        "ignored_test_count": len(ignored_tests),
        "ignored_tests": ignored_tests,
    }
    return inventory, unresolved_markers


def run_generate(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    if not repo_root.is_dir():
        fail(f"repo root is not a directory: {repo_root}")

    inventory, unresolved_markers = generate_inventory(
        repo_root=repo_root,
        scan_roots=args.scan_root,
    )
    output_json = Path(args.output_json)
    if not output_json.is_absolute():
        output_json = (repo_root / output_json).resolve()
    write_json(output_json, inventory)

    if unresolved_markers:
        reason_codes = ["ignored_test_marker_without_function"]
        print("status=fail")
        print(f"ignored_test_count={inventory['ignored_test_count']}")
        print(f"unresolved_marker_count={len(unresolved_markers)}")
        print(f"reason_codes={','.join(reason_codes)}")
        print(f"output_json={output_json}")
        for marker in unresolved_markers:
            print(f"unresolved_marker={marker}", file=sys.stderr)
        return 1

    print("status=generated")
    print(f"ignored_test_count={inventory['ignored_test_count']}")
    print("unresolved_marker_count=0")
    print("reason_codes=none")
    print(f"output_json={output_json}")
    return 0


def load_json_object(path: Path, *, label: str) -> dict[str, Any]:
    if not path.is_file():
        fail(f"{label} not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {label} {path}: {exc}")
    if not isinstance(payload, dict):
        fail(f"{label} must be a JSON object: {path}")
    return payload


def run_check(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    if not repo_root.is_dir():
        fail(f"repo root is not a directory: {repo_root}")

    baseline_path = Path(args.baseline_file)
    if not baseline_path.is_absolute():
        baseline_path = (repo_root / baseline_path).resolve()

    baseline_payload = load_json_object(baseline_path, label="baseline file")
    baseline_ignored_tests = validate_inventory_payload(baseline_payload, label="baseline")

    current_inventory, unresolved_markers = generate_inventory(
        repo_root=repo_root,
        scan_roots=args.scan_root,
    )
    current_ignored_tests = validate_inventory_payload(current_inventory, label="current inventory")

    baseline_set = {
        (entry["source_file"], entry["test_name"])
        for entry in baseline_ignored_tests
    }
    current_set = {
        (entry["source_file"], entry["test_name"])
        for entry in current_ignored_tests
    }

    unexpected_entries = sorted(current_set - baseline_set)
    missing_entries = sorted(baseline_set - current_set)

    reason_codes: list[str] = []
    if unresolved_markers:
        reason_codes.append("ignored_test_marker_without_function")
    if unexpected_entries:
        reason_codes.append("unexpected_ignored_tests_present")
    if missing_entries:
        reason_codes.append("baseline_ignored_tests_missing")

    status = "pass" if not reason_codes else "fail"
    report = {
        "schema_version": DRIFT_REPORT_SCHEMA_VERSION,
        "status": status,
        "repo_root": repo_root.as_posix(),
        "baseline_file": baseline_path.as_posix(),
        "scan_roots": args.scan_root or [DEFAULT_SCAN_ROOT],
        "ignored_test_count": len(current_ignored_tests),
        "baseline_ignored_test_count": len(baseline_ignored_tests),
        "unexpected_entries": [
            {"source_file": source_file, "test_name": test_name}
            for source_file, test_name in unexpected_entries
        ],
        "missing_entries": [
            {"source_file": source_file, "test_name": test_name}
            for source_file, test_name in missing_entries
        ],
        "unresolved_markers": unresolved_markers,
        "reason_codes": reason_codes,
        "violation_count": len(unexpected_entries)
        + len(missing_entries)
        + (len(unresolved_markers) if unresolved_markers else 0),
    }

    output_json = Path(args.output_json)
    if not output_json.is_absolute():
        output_json = (repo_root / output_json).resolve()
    write_json(output_json, report)

    print(f"status={status}")
    print(f"ignored_test_count={len(current_ignored_tests)}")
    print(f"baseline_ignored_test_count={len(baseline_ignored_tests)}")
    print(f"unexpected_count={len(unexpected_entries)}")
    print(f"missing_count={len(missing_entries)}")
    print(f"unresolved_marker_count={len(unresolved_markers)}")
    print(f"violation_count={report['violation_count']}")
    print(f"reason_codes={'none' if not reason_codes else ','.join(reason_codes)}")
    print(f"output_json={output_json}")

    if status == "fail":
        for entry in report["unexpected_entries"]:
            print(
                "unexpected_ignored_test="
                f"{entry['source_file']}::{entry['test_name']}",
                file=sys.stderr,
            )
        for entry in report["missing_entries"]:
            print(
                "missing_baseline_ignored_test="
                f"{entry['source_file']}::{entry['test_name']}",
                file=sys.stderr,
            )
        for marker in unresolved_markers:
            print(f"unresolved_marker={marker}", file=sys.stderr)
        return 1

    return 0


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    try:
        if args.command == "generate":
            return run_generate(args)
        if args.command == "check":
            return run_check(args)
    except CheckerError as error:
        print("status=fail")
        print("reason_codes=checker_configuration_invalid")
        print(f"error={error}")
        return 1

    raise AssertionError(f"unhandled command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
