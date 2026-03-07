"""Git-range helpers for governance/feature commit-ratio CI checks."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Sequence

from governance_feature_commit_ratio_support import (
    Classification,
    CommitRecord,
    HEAD_AT_ACTIVATION_BASE,
    HEAD_PRECEDES_ACTIVATION_BASE,
    POST_ACTIVATION_WINDOW,
    classify_commit_records,
)


class CheckerError(Exception):
    """Raised for deterministic checker failures."""


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


def verify_commit(repo_root: Path, commit_sha: str) -> None:
    run_git(repo_root, ["rev-parse", "--verify", f"{commit_sha}^{{commit}}"])


def is_ancestor(repo_root: Path, ancestor_sha: str, descendant_sha: str) -> bool:
    completed = subprocess.run(
        [
            "git",
            "-C",
            str(repo_root),
            "merge-base",
            "--is-ancestor",
            ancestor_sha,
            descendant_sha,
        ],
        capture_output=True,
        check=False,
        encoding="utf-8",
    )
    if completed.returncode == 0:
        return True
    if completed.returncode == 1:
        return False
    stderr = completed.stderr.strip()
    raise CheckerError(stderr or "git merge-base --is-ancestor failed")


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


def classify_range(
    repo_root: Path,
    base_sha: str,
    head_sha: str,
    window_size: int,
    select_window,
) -> tuple[Classification, int, str, bool]:
    verify_commit(repo_root, base_sha)
    verify_commit(repo_root, head_sha)
    if base_sha == head_sha:
        return Classification(0, 0, ()), 0, HEAD_AT_ACTIVATION_BASE, True
    if is_ancestor(repo_root, head_sha, base_sha):
        return Classification(0, 0, ()), 0, HEAD_PRECEDES_ACTIVATION_BASE, True
    records = read_commit_records(repo_root, base_sha, head_sha)
    return (
        classify_commit_records(select_window(records, window_size)),
        len(records),
        POST_ACTIVATION_WINDOW,
        False,
    )
