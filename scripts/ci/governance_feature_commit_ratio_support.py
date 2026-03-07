"""Shared helpers for governance/feature commit-ratio CI checks."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Sequence

EMPTY_REASON = "governance_commit_subjects_empty"
UNCLASSIFIED_REASON = "governance_commit_subject_unclassified"
THRESHOLD_REASON = "governance_commit_ratio_threshold_exceeded"
GOVERNANCE_PATH_PREFIXES = (
    ".ci/",
    ".github/",
    "docs/ci/",
    "scripts/ci/",
    "specs/",
)
GOVERNANCE_PATHS = frozenset(
    {
        ".github/CONTRIBUTING.md",
        "AGENTS.md",
        "crates/kamn-core/tests/ci_strategy_docs.rs",
    }
)


@dataclass(frozen=True)
class Classification:
    governance_count: int
    feature_count: int
    unknown_subjects: Sequence[str]

    @property
    def non_merge_commit_total(self) -> int:
        return self.governance_count + self.feature_count + len(self.unknown_subjects)


@dataclass(frozen=True)
class CommitRecord:
    subject: str
    paths: Sequence[str]


def serialize_float(value: float) -> float:
    return round(value, 6)


def normalize_path(path: str) -> str:
    return path.strip().lstrip("./")


def is_governance_path(path: str) -> bool:
    normalized = normalize_path(path)
    if not normalized:
        return False
    if normalized in GOVERNANCE_PATHS:
        return True
    return any(normalized.startswith(prefix) for prefix in GOVERNANCE_PATH_PREFIXES)


def classify_commit_records(records: Sequence[CommitRecord]) -> Classification:
    governance_count = 0
    feature_count = 0
    unknown_subjects = []

    for record in records:
        if not record.paths:
            unknown_subjects.append(record.subject)
            continue
        if all(is_governance_path(path) for path in record.paths):
            governance_count += 1
            continue
        feature_count += 1

    return Classification(
        governance_count=governance_count,
        feature_count=feature_count,
        unknown_subjects=tuple(unknown_subjects),
    )


def reason_codes_for_classification(
    classification: Classification,
    max_governance_ratio: float,
) -> Sequence[str]:
    governance_total = classification.governance_count
    feature_total = classification.feature_count
    known_total = governance_total + feature_total
    reason_codes = []
    if classification.non_merge_commit_total == 0:
        reason_codes.append(EMPTY_REASON)
    if classification.unknown_subjects:
        reason_codes.append(UNCLASSIFIED_REASON)
    if known_total > 0 and governance_total / known_total > max_governance_ratio:
        reason_codes.append(THRESHOLD_REASON)
    return tuple(reason_codes)


def build_report(
    classification: Classification,
    input_non_merge_commit_total: int,
    window_size: int,
    max_governance_ratio: float,
    schema_version: str,
    reason_taxonomy_version: str,
    governance_types_csv: str,
    feature_types_csv: str,
) -> Dict[str, object]:
    governance_count = classification.governance_count
    feature_count = classification.feature_count
    known_total = governance_count + feature_count
    governance_ratio = 0.0
    feature_ratio = 0.0
    if known_total > 0:
        governance_ratio = governance_count / known_total
        feature_ratio = feature_count / known_total
    reason_codes = reason_codes_for_classification(classification, max_governance_ratio)
    return {
        "schema_version": schema_version,
        "reason_taxonomy_version": reason_taxonomy_version,
        "reason_codes_csv": "none" if not reason_codes else ",".join(reason_codes),
        "status": "ok" if not reason_codes else "violation",
        "input_non_merge_commit_total": input_non_merge_commit_total,
        "non_merge_commit_total": classification.non_merge_commit_total,
        "governance_commit_count": governance_count,
        "feature_commit_count": feature_count,
        "unknown_commit_count": len(classification.unknown_subjects),
        "window_size": window_size,
        "max_governance_ratio": serialize_float(max_governance_ratio),
        "governance_ratio": serialize_float(governance_ratio),
        "feature_ratio": serialize_float(feature_ratio),
        "governance_commit_types_csv": governance_types_csv,
        "feature_commit_types_csv": feature_types_csv,
        "unknown_commit_subjects": list(classification.unknown_subjects),
    }


def build_error_report(
    error: str,
    max_governance_ratio: float,
    window_size: int,
    schema_version: str,
    reason_taxonomy_version: str,
) -> Dict[str, object]:
    return {
        "schema_version": schema_version,
        "reason_taxonomy_version": reason_taxonomy_version,
        "reason_codes_csv": EMPTY_REASON,
        "status": "violation",
        "error": error,
        "input_non_merge_commit_total": 0,
        "non_merge_commit_total": 0,
        "governance_commit_count": 0,
        "feature_commit_count": 0,
        "unknown_commit_count": 0,
        "window_size": window_size,
        "max_governance_ratio": serialize_float(max_governance_ratio),
        "governance_ratio": 0.0,
        "feature_ratio": 0.0,
    }


def emit_stdout(report: Dict[str, object]) -> None:
    print(f"status={report['status']}")
    print(f"reason_taxonomy_version={report['reason_taxonomy_version']}")
    print(f"reason_codes_csv={report['reason_codes_csv']}")
    print(f"input_non_merge_commit_total={report['input_non_merge_commit_total']}")
    print(f"non_merge_commit_total={report['non_merge_commit_total']}")
    print(f"governance_commit_count={report['governance_commit_count']}")
    print(f"feature_commit_count={report['feature_commit_count']}")
    print(f"unknown_commit_count={report['unknown_commit_count']}")
    print(f"window_size={report['window_size']}")
    print(f"max_governance_ratio={report['max_governance_ratio']}")
    print(f"governance_ratio={report['governance_ratio']}")
    print(f"feature_ratio={report['feature_ratio']}")
