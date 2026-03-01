#!/usr/bin/env python3
"""Fail-closed governance/feature commit-ratio checker for CI gates."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Sequence

SCHEMA_VERSION = "kamn.ci.governance-feature-commit-ratio-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.governance-feature-commit-ratio-reason-taxonomy.v1"
REASON_CODES = [
    "governance_commit_subjects_empty",
    "governance_commit_subject_unclassified",
    "governance_commit_ratio_threshold_exceeded",
]

GOVERNANCE_TYPES = frozenset({"spec", "docs", "chore"})
FEATURE_TYPES = frozenset({"feat", "fix", "refactor", "test", "perf", "integrate"})


class CheckerError(Exception):
    """Raised for deterministic checker failures."""


@dataclass(frozen=True)
class Classification:
    governance_count: int
    feature_count: int
    unknown_subjects: Sequence[str]

    @property
    def non_merge_commit_total(self) -> int:
        return self.governance_count + self.feature_count + len(self.unknown_subjects)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce governance/feature commit ratio threshold on PR commit subjects."
    )
    parser.add_argument(
        "--commit-subjects-file",
        required=True,
        help="Path to newline-delimited commit subject list.",
    )
    parser.add_argument(
        "--max-governance-ratio",
        type=float,
        default=0.50,
        help="Maximum allowed governance ratio (0.0..1.0).",
    )
    parser.add_argument(
        "--output-json",
        default="ci-governance-feature-commit-ratio.json",
        help="Output path for schema-versioned JSON report.",
    )
    return parser.parse_args()


def read_commit_subjects(path: Path) -> List[str]:
    if not path.exists():
        raise CheckerError(f"commit subjects file not found: {path}")
    contents = path.read_text(encoding="utf-8")
    return [line.strip() for line in contents.splitlines() if line.strip()]


def commit_type_from_subject(subject: str) -> str | None:
    head = subject.split(":", 1)[0].strip()
    if not head:
        return None
    if "(" in head:
        prefix = head.split("(", 1)[0].strip().lower()
    else:
        prefix = head.strip().lower()
    if not prefix:
        return None
    return prefix


def classify_subjects(subjects: Iterable[str]) -> Classification:
    governance_count = 0
    feature_count = 0
    unknown_subjects: List[str] = []

    for subject in subjects:
        commit_type = commit_type_from_subject(subject)
        if commit_type in GOVERNANCE_TYPES:
            governance_count += 1
            continue
        if commit_type in FEATURE_TYPES:
            feature_count += 1
            continue
        unknown_subjects.append(subject)

    return Classification(
        governance_count=governance_count,
        feature_count=feature_count,
        unknown_subjects=tuple(unknown_subjects),
    )


def serialize_float(value: float) -> float:
    return round(value, 6)


def build_report(
    classification: Classification,
    max_governance_ratio: float,
) -> Dict[str, object]:
    governance_count = classification.governance_count
    feature_count = classification.feature_count
    known_total = governance_count + feature_count
    governance_ratio = 0.0
    feature_ratio = 0.0
    if known_total > 0:
        governance_ratio = governance_count / known_total
        feature_ratio = feature_count / known_total

    reason_codes: List[str] = []
    if classification.non_merge_commit_total == 0:
        reason_codes.append("governance_commit_subjects_empty")
    if classification.unknown_subjects:
        reason_codes.append("governance_commit_subject_unclassified")
    if known_total > 0 and governance_ratio > max_governance_ratio:
        reason_codes.append("governance_commit_ratio_threshold_exceeded")

    status = "ok" if not reason_codes else "violation"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    return {
        "schema_version": SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": reason_codes_csv,
        "status": status,
        "non_merge_commit_total": classification.non_merge_commit_total,
        "governance_commit_count": governance_count,
        "feature_commit_count": feature_count,
        "unknown_commit_count": len(classification.unknown_subjects),
        "max_governance_ratio": serialize_float(max_governance_ratio),
        "governance_ratio": serialize_float(governance_ratio),
        "feature_ratio": serialize_float(feature_ratio),
        "governance_commit_types_csv": ",".join(sorted(GOVERNANCE_TYPES)),
        "feature_commit_types_csv": ",".join(sorted(FEATURE_TYPES)),
        "unknown_commit_subjects": list(classification.unknown_subjects),
    }


def write_report(path: Path, report: Dict[str, object]) -> None:
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def emit_stdout(report: Dict[str, object]) -> None:
    print(f"status={report['status']}")
    print(f"reason_taxonomy_version={report['reason_taxonomy_version']}")
    print(f"reason_codes_csv={report['reason_codes_csv']}")
    print(f"non_merge_commit_total={report['non_merge_commit_total']}")
    print(f"governance_commit_count={report['governance_commit_count']}")
    print(f"feature_commit_count={report['feature_commit_count']}")
    print(f"unknown_commit_count={report['unknown_commit_count']}")
    print(f"max_governance_ratio={report['max_governance_ratio']}")
    print(f"governance_ratio={report['governance_ratio']}")
    print(f"feature_ratio={report['feature_ratio']}")


def validate_args(args: argparse.Namespace) -> None:
    if args.max_governance_ratio < 0.0 or args.max_governance_ratio > 1.0:
        raise CheckerError("--max-governance-ratio must be within [0.0, 1.0]")


def main() -> int:
    args = parse_args()
    try:
        validate_args(args)
        subjects = read_commit_subjects(Path(args.commit_subjects_file))
        classification = classify_subjects(subjects)
        report = build_report(classification, float(args.max_governance_ratio))
        write_report(Path(args.output_json), report)
        emit_stdout(report)
        return 0 if report["status"] == "ok" else 1
    except CheckerError as error:
        report = {
            "schema_version": SCHEMA_VERSION,
            "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
            "reason_codes_csv": "governance_commit_subjects_empty",
            "status": "violation",
            "error": str(error),
        }
        write_report(Path(args.output_json), report)
        emit_stdout(report)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
