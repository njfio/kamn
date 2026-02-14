#!/usr/bin/env python3
"""Fail-closed drift guard for legacy synchronous ingress parser markers."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
import json
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "kamn.ci.legacy-ingress-parser-baseline.v1"


@dataclass(frozen=True)
class MarkerRule:
    marker_id: str
    pattern: str
    max_occurrences: int
    allowed_files: tuple[str, ...]


@dataclass(frozen=True)
class BaselineConfig:
    source_root: Path
    exclude_path_fragments: tuple[str, ...]
    marker_rules: tuple[MarkerRule, ...]


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check for legacy synchronous ingress parser drift."
    )
    parser.add_argument(
        "--source-root",
        default="crates/kamn-node/src",
        help="Source root to scan for Rust files.",
    )
    parser.add_argument(
        "--baseline-file",
        default="fixtures/ci/legacy_ingress_parser_baseline.json",
        help="Baseline contract file path.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output JSON report path.",
    )
    return parser.parse_args(argv)


def _load_baseline(path: Path, source_root: Path) -> BaselineConfig:
    if not path.is_file():
        raise ValueError("legacy_ingress_parser_baseline_missing")

    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError("legacy_ingress_parser_baseline_invalid") from exc

    if not isinstance(payload, dict):
        raise ValueError("legacy_ingress_parser_baseline_invalid")
    if payload.get("schema_version") != SCHEMA_VERSION:
        raise ValueError("legacy_ingress_parser_baseline_invalid")

    raw_excludes = payload.get("exclude_path_fragments", [])
    if not isinstance(raw_excludes, list) or any(
        not isinstance(item, str) or item.strip() == "" for item in raw_excludes
    ):
        raise ValueError("legacy_ingress_parser_baseline_invalid")
    excludes = tuple(raw_excludes)

    raw_markers = payload.get("markers")
    if not isinstance(raw_markers, list) or len(raw_markers) == 0:
        raise ValueError("legacy_ingress_parser_baseline_invalid")

    marker_rules: list[MarkerRule] = []
    for item in raw_markers:
        if not isinstance(item, dict):
            raise ValueError("legacy_ingress_parser_baseline_invalid")

        marker_id = item.get("id")
        pattern = item.get("pattern")
        max_occurrences = item.get("max_occurrences")
        allowed_files = item.get("allowed_files")

        if not isinstance(marker_id, str) or marker_id.strip() == "":
            raise ValueError("legacy_ingress_parser_baseline_invalid")
        if not isinstance(pattern, str) or pattern.strip() == "":
            raise ValueError("legacy_ingress_parser_baseline_invalid")
        if not isinstance(max_occurrences, int) or max_occurrences < 0:
            raise ValueError("legacy_ingress_parser_baseline_invalid")
        if not isinstance(allowed_files, list) or len(allowed_files) == 0:
            raise ValueError("legacy_ingress_parser_baseline_invalid")

        normalized_allowed: list[str] = []
        for raw_file in allowed_files:
            if not isinstance(raw_file, str) or raw_file.strip() == "":
                raise ValueError("legacy_ingress_parser_baseline_invalid")
            normalized_allowed.append(raw_file.replace("\\", "/"))

        marker_rules.append(
            MarkerRule(
                marker_id=marker_id,
                pattern=pattern,
                max_occurrences=max_occurrences,
                allowed_files=tuple(normalized_allowed),
            )
        )

    return BaselineConfig(
        source_root=source_root,
        exclude_path_fragments=excludes,
        marker_rules=tuple(marker_rules),
    )


def _collect_rs_files(source_root: Path, exclude_fragments: tuple[str, ...]) -> list[Path]:
    files: list[Path] = []
    for path in sorted(source_root.rglob("*.rs")):
        rel = path.relative_to(source_root).as_posix()
        if any(fragment in rel for fragment in exclude_fragments):
            continue
        files.append(path)
    return files


def _build_report(
    source_root: Path,
    baseline_file: Path,
    baseline: BaselineConfig,
) -> dict[str, Any]:
    rs_files = _collect_rs_files(source_root, baseline.exclude_path_fragments)

    marker_totals: dict[str, int] = {}
    marker_file_counts: dict[str, dict[str, int]] = {}
    count_increase_markers: list[str] = []
    new_file_markers: list[str] = []

    for rule in baseline.marker_rules:
        total = 0
        file_counts: dict[str, int] = {}
        for path in rs_files:
            rel = path.relative_to(source_root).as_posix()
            text = path.read_text(encoding="utf-8")
            count = text.count(rule.pattern)
            if count <= 0:
                continue
            total += count
            file_counts[rel] = count
            if rel not in rule.allowed_files and rule.marker_id not in new_file_markers:
                new_file_markers.append(rule.marker_id)

        marker_totals[rule.marker_id] = total
        marker_file_counts[rule.marker_id] = file_counts
        if total > rule.max_occurrences:
            count_increase_markers.append(rule.marker_id)

    reason_codes: list[str] = []
    if count_increase_markers:
        reason_codes.append("legacy_ingress_parser_marker_count_increased")
    if new_file_markers:
        reason_codes.append("legacy_ingress_parser_marker_new_file")

    status = "pass" if not reason_codes else "fail"
    policy_decision = "GO" if status == "pass" else "NO-GO"

    if not reason_codes:
        reason_codes = ["none"]

    remediation = (
        "none"
        if status == "pass"
        else (
            "reduce legacy parser markers to baseline constraints in "
            f"{baseline_file} or remove legacy sync ingress paths from service runtime"
        )
    )

    return {
        "schema_version": "kamn.ci.legacy-ingress-parser-drift-report.v1",
        "status": status,
        "policy_decision": policy_decision,
        "reason_codes": reason_codes,
        "source_root": str(source_root),
        "baseline_file": str(baseline_file),
        "exclude_path_fragments": list(baseline.exclude_path_fragments),
        "marker_totals": marker_totals,
        "marker_file_counts": marker_file_counts,
        "count_increase_markers": sorted(count_increase_markers),
        "new_file_markers": sorted(new_file_markers),
        "remediation": remediation,
    }


def _print_report(report: dict[str, Any]) -> None:
    reason_codes = report["reason_codes"]
    count_increase = report["count_increase_markers"]
    new_file = report["new_file_markers"]

    print(f"status={report['status']}")
    print(f"policy_decision={report['policy_decision']}")
    print(
        "reason_codes="
        + ("none" if reason_codes == ["none"] else ",".join(sorted(reason_codes)))
    )
    print(
        "count_increase_markers="
        + ("none" if not count_increase else ",".join(sorted(count_increase)))
    )
    print(
        "new_file_markers="
        + ("none" if not new_file else ",".join(sorted(new_file)))
    )
    print(f"source_root={report['source_root']}")
    print(f"baseline_file={report['baseline_file']}")
    print(f"remediation={report['remediation']}")


def _print_error_report(
    *,
    reason_code: str,
    source_root: Path,
    baseline_file: Path,
) -> None:
    print("status=fail")
    print("policy_decision=NO-GO")
    print(f"reason_codes={reason_code}")
    print("count_increase_markers=none")
    print("new_file_markers=none")
    print(f"source_root={source_root}")
    print(f"baseline_file={baseline_file}")
    print(
        "remediation=repair baseline schema/file availability before enforcing legacy ingress parser drift checks"
    )


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    source_root = Path(args.source_root)
    baseline_file = Path(args.baseline_file)

    if not source_root.is_dir():
        _print_error_report(
            reason_code="legacy_ingress_parser_source_root_missing",
            source_root=source_root,
            baseline_file=baseline_file,
        )
        return 1

    try:
        baseline = _load_baseline(baseline_file, source_root)
    except ValueError as exc:
        reason_code = str(exc)
        _print_error_report(
            reason_code=reason_code,
            source_root=source_root,
            baseline_file=baseline_file,
        )
        report = {
            "schema_version": "kamn.ci.legacy-ingress-parser-drift-report.v1",
            "status": "fail",
            "policy_decision": "NO-GO",
            "reason_codes": [reason_code],
            "source_root": str(source_root),
            "baseline_file": str(baseline_file),
            "exclude_path_fragments": [],
            "marker_totals": {},
            "marker_file_counts": {},
            "count_increase_markers": [],
            "new_file_markers": [],
            "remediation": "repair baseline schema/file availability before enforcing legacy ingress parser drift checks",
        }
        if args.output_json:
            output_path = Path(args.output_json)
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
            print(f"report_file={output_path}")
        return 1

    report = _build_report(
        source_root=source_root,
        baseline_file=baseline_file,
        baseline=baseline,
    )
    _print_report(report)

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")
        print(f"report_file={output_path}")

    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main(__import__("sys").argv[1:]))
