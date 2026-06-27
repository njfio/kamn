#!/usr/bin/env python3
"""Enforce ratcheted Rust file/function size policy on touched code."""

from __future__ import annotations

import argparse
from pathlib import Path
import subprocess
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.ci.touched_rust_size_policy_support import (
    PolicyError,
    PolicyReport,
    extract_function_spans,
    read_json,
    validate_baseline_payload,
    validate_threshold_payload,
)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', default='.', help='Repository root.')
    parser.add_argument('--base-ref', default='', help='Base ref for merge-base lookup.')
    parser.add_argument('--threshold-file', default='fixtures/ci/touched_rust_size_policy_thresholds.json')
    parser.add_argument('--baseline-file', default='fixtures/ci/touched_rust_size_policy_baseline.json')
    parser.add_argument('--output-json', required=True, help='Output report path.')
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo_root = Path(args.repo_root).resolve()
    try:
        file_limit, function_limit = load_limits(repo_root, args.threshold_file)
        baseline_files, baseline_functions = load_baseline(repo_root, args.baseline_file)
        merge_base = resolve_merge_base(repo_root, args.base_ref)
    except PolicyError as error:
        return error.emit(args.output_json)
    touched = changed_rust_files(repo_root, merge_base)
    report = evaluate_touched_files(repo_root, merge_base, touched, file_limit, function_limit, baseline_files, baseline_functions)
    return report.emit(args.output_json)


def load_limits(repo_root: Path, threshold_file: str) -> tuple[int, int]:
    path = resolve_path(repo_root, threshold_file)
    try:
        return validate_threshold_payload(read_json(path))
    except Exception as error:
        raise PolicyError('touched_rust_size_policy_threshold_invalid', str(error)) from error


def load_baseline(repo_root: Path, baseline_file: str) -> tuple[dict[str, int], dict[tuple[str, str], int]]:
    path = resolve_path(repo_root, baseline_file)
    try:
        return validate_baseline_payload(read_json(path))
    except Exception as error:
        raise PolicyError('touched_rust_size_policy_baseline_invalid', str(error)) from error


def resolve_path(repo_root: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else (repo_root / path).resolve()


def resolve_merge_base(repo_root: Path, base_ref: str) -> str:
    for candidate in merge_base_candidates(base_ref):
        if _git(repo_root, 'rev-parse', '--verify', candidate).returncode != 0:
            continue
        merge_base = _git(repo_root, 'merge-base', 'HEAD', candidate)
        if merge_base.returncode == 0 and merge_base.stdout.strip():
            return merge_base.stdout.strip()
    raise PolicyError('touched_rust_size_policy_git_base_unavailable', 'unable to resolve merge-base')


def merge_base_candidates(base_ref: str) -> list[str]:
    if not base_ref:
        return ['origin/main', 'main', 'HEAD~1']
    if base_ref.startswith('origin/'):
        return [base_ref]
    return [f'origin/{base_ref}', base_ref]


def changed_rust_files(repo_root: Path, merge_base: str) -> list[str]:
    result = _git(repo_root, 'diff', '--name-only', '--diff-filter=AMR', merge_base, '--', 'crates')
    return sorted(filter_changed_paths(repo_root, result.stdout.splitlines()))


def filter_changed_paths(repo_root: Path, paths: list[str]) -> list[str]:
    return [path.strip() for path in paths if path.strip().endswith('.rs') and (repo_root / path.strip()).is_file()]


def evaluate_touched_files(
    repo_root: Path,
    merge_base: str,
    touched: list[str],
    file_limit: int,
    function_limit: int,
    baseline_files: dict[str, int],
    baseline_functions: dict[tuple[str, str], int],
) -> PolicyReport:
    offending_files: list[str] = []
    offending_functions: list[str] = []
    for rel_path in touched:
        current, previous = load_source_pair(repo_root, merge_base, rel_path)
        if file_regression(rel_path, current, previous, file_limit, baseline_files):
            offending_files.append(rel_path)
            continue
        offending_functions.extend(
            function_regressions(rel_path, current, previous, function_limit, baseline_functions)
        )
    return build_report(merge_base, touched, offending_files, offending_functions)


def build_report(
    merge_base: str,
    touched: list[str],
    offending_files: list[str],
    offending_functions: list[str],
) -> PolicyReport:
    touched_csv = ','.join(touched) if touched else 'none'
    if offending_files:
        return PolicyReport(
            'touched_rust_size_policy_new_oversized_file',
            'split touched oversized files below the file limit or land a separate remediation issue first',
            merge_base=merge_base,
            touched_rust_files=touched_csv,
            offending_files=','.join(offending_files),
        )
    if offending_functions:
        return PolicyReport(
            'touched_rust_size_policy_new_oversized_function',
            'extract touched oversized functions below the function limit before merge',
            merge_base=merge_base,
            touched_rust_files=touched_csv,
            offending_functions=','.join(offending_functions),
        )
    return PolicyReport('none', 'none', merge_base=merge_base, touched_rust_files=touched_csv)


def load_source_pair(repo_root: Path, merge_base: str, rel_path: str) -> tuple[str, str | None]:
    return (repo_root / rel_path).read_text(encoding='utf-8'), git_show_text(repo_root, merge_base, rel_path)


def file_regression(
    rel_path: str,
    current: str,
    previous: str | None,
    limit: int,
    baseline_files: dict[str, int],
) -> bool:
    current_lines = len(current.splitlines())
    if current_lines <= limit:
        return False
    if baseline_files.get(rel_path) == current_lines:
        return False
    previous_lines = len(previous.splitlines()) if previous is not None else 0
    return previous_lines <= limit


def function_regressions(
    rel_path: str,
    current: str,
    previous: str | None,
    limit: int,
    baseline_functions: dict[tuple[str, str], int],
) -> list[str]:
    previous_spans = map_spans(rel_path, previous) if previous is not None else {}
    offenders: list[str] = []
    for span in extract_function_spans(rel_path, current):
        if span.line_count <= limit:
            continue
        if baseline_functions.get((rel_path, span.header_key)) == span.line_count:
            continue
        previous_lines = previous_spans.get(span.header_key, 0)
        if previous_lines <= limit:
            offenders.append(span.offender_id())
    return offenders


def map_spans(rel_path: str, text: str) -> dict[str, int]:
    return {span.header_key: span.line_count for span in extract_function_spans(rel_path, text)}


def git_show_text(repo_root: Path, commit: str, rel_path: str) -> str | None:
    shown = _git(repo_root, 'show', f'{commit}:{rel_path}')
    return shown.stdout if shown.returncode == 0 else None


def _git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(['git', '-C', str(repo_root), *args], check=False, text=True, capture_output=True)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
