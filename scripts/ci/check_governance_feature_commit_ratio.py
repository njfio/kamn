#!/usr/bin/env python3
"""Fail-closed governance/feature commit-ratio checker for CI gates."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
from typing import Dict, Iterable, List, Sequence

from governance_feature_commit_ratio_support import (
    Classification,
    CommitRecord,
    build_error_report,
    build_report,
    classify_commit_records,
    emit_stdout,
)

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


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Enforce governance/feature commit ratio threshold on PR commit subjects."
    )
    parser.add_argument(
        "--commit-subjects-file",
        help="Path to newline-delimited commit subject list.",
    )
    parser.add_argument(
        "--repo-root",
        help="Repository root for git-range classification mode.",
    )
    parser.add_argument(
        "--base-sha",
        help="Inclusive moratorium cutoff SHA for git-range classification mode.",
    )
    parser.add_argument(
        "--head-sha",
        help="Head SHA to evaluate in git-range classification mode.",
    )
    parser.add_argument(
        "--window-size",
        type=int,
        default=0,
        help="Maximum number of newest commit subjects to evaluate; 0 means all input subjects.",
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


def select_window(items: Sequence[object], window_size: int) -> Sequence[object]:
    if window_size <= 0 or window_size >= len(items):
        return tuple(items)
    return tuple(items[:window_size])


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


def run_git(repo_root: Path, args: Sequence[str]) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repo_root), *args],
        capture_output=True,
        check=False,
        encoding="utf-8",
    )
    if completed.returncode != 0:
        stderr = completed.stderr.strip()
        raise CheckerError(stderr or f"git command failed: {' '.join(args)}")
    return completed.stdout


def read_commit_records(repo_root: Path, base_sha: str, head_sha: str) -> Sequence[CommitRecord]:
    if not repo_root.exists():
        raise CheckerError(f"repo root not found: {repo_root}")
    commit_shas = [
        line.strip()
        for line in run_git(repo_root, ["rev-list", "--no-merges", f"{base_sha}..{head_sha}"]).splitlines()
        if line.strip()
    ]
    records = []
    for commit_sha in commit_shas:
        subject = run_git(repo_root, ["show", "-s", "--format=%s", commit_sha]).strip()
        paths = [
            line.strip()
            for line in run_git(
                repo_root, ["diff-tree", "--no-commit-id", "--name-only", "-r", commit_sha]
            ).splitlines()
            if line.strip()
        ]
        records.append(CommitRecord(subject=subject, paths=tuple(paths)))
    return tuple(records)


def write_report(path: Path, report: Dict[str, object]) -> None:
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def validate_args(args: argparse.Namespace) -> None:
    if args.max_governance_ratio < 0.0 or args.max_governance_ratio > 1.0:
        raise CheckerError("--max-governance-ratio must be within [0.0, 1.0]")
    if args.window_size < 0:
        raise CheckerError("--window-size must be >= 0")
    if args.commit_subjects_file:
        return
    if not args.repo_root or not args.base_sha or not args.head_sha:
        raise CheckerError(
            "either --commit-subjects-file or the full --repo-root/--base-sha/--head-sha set is required"
        )


def main() -> int:
    args = parse_args()
    try:
        validate_args(args)
        input_total = 0
        if args.commit_subjects_file:
            subjects = read_commit_subjects(Path(args.commit_subjects_file))
            classification = classify_subjects(select_window(subjects, int(args.window_size)))
            input_total = len(subjects)
        else:
            records = read_commit_records(Path(args.repo_root), str(args.base_sha), str(args.head_sha))
            classification = classify_commit_records(select_window(records, int(args.window_size)))
            input_total = len(records)
        report = build_report(
            classification,
            input_non_merge_commit_total=input_total,
            window_size=int(args.window_size),
            max_governance_ratio=float(args.max_governance_ratio),
            schema_version=SCHEMA_VERSION,
            reason_taxonomy_version=REASON_TAXONOMY_VERSION,
            governance_types_csv=",".join(sorted(GOVERNANCE_TYPES)),
            feature_types_csv=",".join(sorted(FEATURE_TYPES)),
        )
        write_report(Path(args.output_json), report)
        emit_stdout(report)
        return 0 if report["status"] == "ok" else 1
    except CheckerError as error:
        report = build_error_report(
            str(error),
            max_governance_ratio=float(getattr(args, "max_governance_ratio", 0.0)),
            window_size=int(getattr(args, "window_size", 0)),
            schema_version=SCHEMA_VERSION,
            reason_taxonomy_version=REASON_TAXONOMY_VERSION,
        )
        write_report(Path(args.output_json), report)
        emit_stdout(report)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
