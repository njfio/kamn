#!/usr/bin/env python3
"""Enforce ratcheted Rust file/function size policy on touched code."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))

from scripts.ci.touched_rust_size_policy_support import (
    extract_function_spans,
    validate_baseline_payload,
    validate_threshold_payload,
)

OUTPUT_SCHEMA = 'kamn.ci.touched-rust-size-policy-report.v1'


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', default='.', help='Repository root.')
    parser.add_argument('--base-ref', default='', help='Base ref for merge-base lookup.')
    parser.add_argument(
        '--threshold-file',
        default='fixtures/ci/touched_rust_size_policy_thresholds.json',
        help='Threshold JSON file.',
    )
    parser.add_argument(
        '--baseline-file',
        default='fixtures/ci/touched_rust_size_policy_baseline.json',
        help='Committed baseline inventory JSON file.',
    )
    parser.add_argument('--output-json', required=True, help='Output report path.')
    return parser.parse_args(argv)


def _git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ['git', '-C', str(repo_root), *args],
        check=False,
        text=True,
        capture_output=True,
    )


def _print(status: str, decision: str, reason: str, report: dict[str, Any]) -> int:
    payload = {'schema_version': OUTPUT_SCHEMA, 'status': status, 'policy_decision': decision, **report}
    output_path = Path(report['output_json'])
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    print(f'status={status}')
    print(f'policy_decision={decision}')
    print(f'merge_base={report["merge_base"]}')
    print(f'touched_rust_files={report["touched_rust_files"]}')
    print(f'offending_files={report["offending_files"]}')
    print(f'offending_functions={report["offending_functions"]}')
    print(f'reason_codes={reason}')
    print(f'remediation={report["remediation"]}')
    return 0 if decision == 'GO' else 1


def fail(reason: str, remediation: str, output_json: str, **report: Any) -> int:
    merged = {
        'merge_base': report.get('merge_base', 'none'),
        'touched_rust_files': report.get('touched_rust_files', 'none'),
        'offending_files': report.get('offending_files', 'none'),
        'offending_functions': report.get('offending_functions', 'none'),
        'remediation': remediation,
        'output_json': output_json,
    }
    return _print('fail', 'NO-GO', reason, merged)


def pass_report(output_json: str, merge_base: str, touched: list[str]) -> int:
    joined = ','.join(touched) if touched else 'none'
    return _print(
        'pass',
        'GO',
        'none',
        {
            'merge_base': merge_base,
            'touched_rust_files': joined,
            'offending_files': 'none',
            'offending_functions': 'none',
            'remediation': 'none',
            'output_json': output_json,
        },
    )


def resolve_merge_base(repo_root: Path, base_ref: str) -> str:
    candidates = [base_ref] if base_ref else []
    if base_ref and not base_ref.startswith('origin/'):
        candidates.insert(0, f'origin/{base_ref}')
    if not candidates:
        candidates = ['origin/main', 'main', 'HEAD~1']
    for candidate in candidates:
        if _git(repo_root, 'rev-parse', '--verify', candidate).returncode != 0:
            continue
        merge_base = _git(repo_root, 'merge-base', 'HEAD', candidate)
        if merge_base.returncode == 0 and merge_base.stdout.strip():
            return merge_base.stdout.strip()
    raise ValueError('unable to resolve merge-base')


def changed_rust_files(repo_root: Path, merge_base: str) -> list[str]:
    result = _git(repo_root, 'diff', '--name-only', '--diff-filter=AMR', merge_base, '--', 'crates')
    paths = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    return sorted(
        path
        for path in paths
        if path.endswith('.rs') and (repo_root / path).is_file()
    )


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding='utf-8'))


def git_show_text(repo_root: Path, commit: str, rel_path: str) -> str | None:
    shown = _git(repo_root, 'show', f'{commit}:{rel_path}')
    return shown.stdout if shown.returncode == 0 else None


def file_regression(current: str, previous: str | None, limit: int) -> bool:
    current_lines = len(current.splitlines())
    previous_lines = len(previous.splitlines()) if previous is not None else 0
    return current_lines > limit and previous_lines <= limit


def function_regressions(rel_path: str, current: str, previous: str | None, limit: int) -> list[str]:
    previous_spans = {}
    if previous is not None:
        previous_spans = {span.header_key: span for span in extract_function_spans(rel_path, previous)}
    offenders: list[str] = []
    for span in extract_function_spans(rel_path, current):
        previous_lines = previous_spans.get(span.header_key).line_count if span.header_key in previous_spans else 0
        if span.line_count > limit and previous_lines <= limit:
            offenders.append(span.offender_id())
    return offenders


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo_root = Path(args.repo_root).resolve()
    threshold_path = (repo_root / args.threshold_file).resolve() if not Path(args.threshold_file).is_absolute() else Path(args.threshold_file)
    baseline_path = (repo_root / args.baseline_file).resolve() if not Path(args.baseline_file).is_absolute() else Path(args.baseline_file)
    try:
        file_limit, function_limit = validate_threshold_payload(read_json(threshold_path))
    except Exception as error:
        return fail('touched_rust_size_policy_threshold_invalid', str(error), args.output_json)
    try:
        validate_baseline_payload(read_json(baseline_path))
    except Exception as error:
        return fail('touched_rust_size_policy_baseline_invalid', str(error), args.output_json)
    try:
        merge_base = resolve_merge_base(repo_root, args.base_ref)
    except ValueError as error:
        return fail('touched_rust_size_policy_git_base_unavailable', str(error), args.output_json)
    touched = changed_rust_files(repo_root, merge_base)
    offending_files: list[str] = []
    offending_functions: list[str] = []
    for rel_path in touched:
        current_text = (repo_root / rel_path).read_text(encoding='utf-8')
        previous_text = git_show_text(repo_root, merge_base, rel_path)
        if file_regression(current_text, previous_text, file_limit):
            offending_files.append(rel_path)
            continue
        offending_functions.extend(function_regressions(rel_path, current_text, previous_text, function_limit))
    if offending_files:
        return fail(
            'touched_rust_size_policy_new_oversized_file',
            'split touched oversized files below the file limit or land a separate remediation issue first',
            args.output_json,
            merge_base=merge_base,
            touched_rust_files=','.join(touched) if touched else 'none',
            offending_files=','.join(offending_files),
            offending_functions='none',
        )
    if offending_functions:
        return fail(
            'touched_rust_size_policy_new_oversized_function',
            'extract touched oversized functions below the function limit before merge',
            args.output_json,
            merge_base=merge_base,
            touched_rust_files=','.join(touched) if touched else 'none',
            offending_files='none',
            offending_functions=','.join(offending_functions),
        )
    return pass_report(args.output_json, merge_base, touched)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
