"""Shared helpers for governance/feature commit-ratio CI checks."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Sequence


@dataclass(frozen=True)
class Classification:
    governance_count: int
    feature_count: int
    unknown_subjects: Sequence[str]

    @property
    def non_merge_commit_total(self) -> int:
        return self.governance_count + self.feature_count + len(self.unknown_subjects)


def serialize_float(value: float) -> float:
    return round(value, 6)


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
    reason_codes = []
    if classification.non_merge_commit_total == 0:
        reason_codes.append("governance_commit_subjects_empty")
    if classification.unknown_subjects:
        reason_codes.append("governance_commit_subject_unclassified")
    if known_total > 0 and governance_ratio > max_governance_ratio:
        reason_codes.append("governance_commit_ratio_threshold_exceeded")
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
        "reason_codes_csv": "governance_commit_subjects_empty",
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
