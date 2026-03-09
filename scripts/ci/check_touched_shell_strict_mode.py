#!/usr/bin/env python3
"""Enforce strict mode on touched executable shell scripts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo-root', default='.', help='Repository root.')
    parser.add_argument('--base-ref', default='', help='Base ref for merge-base lookup.')
    parser.add_argument(
        '--exception-file',
        default='fixtures/ci/touched_shell_strict_mode_exceptions.txt',
        help='Newline-delimited relative paths exempt from strict-mode enforcement.',
    )
    parser.add_argument('--output-json', required=True, help='Output report path.')
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo_root = Path(args.repo_root).resolve()
    try:
        merge_base = resolve_merge_base(repo_root, args.base_ref)
        exceptions = load_exceptions(repo_root, args.exception_file)
    except PolicyError as error:
        return error.emit(args.output_json)
    touched = changed_shell_files(repo_root, merge_base)
    report = evaluate(repo_root, merge_base, touched, exceptions)
    return emit_report(args.output_json, report)


def resolve_merge_base(repo_root: Path, base_ref: str) -> str:
    for candidate in merge_base_candidates(base_ref):
        if git(repo_root, 'rev-parse', '--verify', candidate).returncode != 0:
            continue
        result = git(repo_root, 'merge-base', 'HEAD', candidate)
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip()
    raise PolicyError(
        'touched_shell_strict_mode_git_base_unavailable',
        'unable to resolve merge-base',
    )


def merge_base_candidates(base_ref: str) -> list[str]:
    if not base_ref:
        return ['origin/main', 'main', 'HEAD~1']
    return [base_ref] if base_ref.startswith('origin/') else [f'origin/{base_ref}', base_ref]


def load_exceptions(repo_root: Path, value: str) -> set[str]:
    path = resolve_path(repo_root, value)
    try:
        return parse_exceptions(repo_root, path)
    except Exception as error:
        raise PolicyError('touched_shell_strict_mode_exception_file_invalid', str(error)) from error


def parse_exceptions(repo_root: Path, path: Path) -> set[str]:
    entries: set[str] = set()
    for raw_line in path.read_text(encoding='utf-8').splitlines():
        line = raw_line.strip()
        if not line or line.startswith('#'):
            continue
        validate_exception_entry(repo_root, line, entries)
        entries.add(line)
    return entries


def validate_exception_entry(repo_root: Path, line: str, entries: set[str]) -> None:
    if line in entries:
        raise ValueError(f'duplicate exception entry: {line}')
    if not line.startswith('scripts/') or not line.endswith('.sh'):
        raise ValueError(f'invalid exception path: {line}')
    if not (repo_root / line).is_file():
        raise ValueError(f'exception path does not exist: {line}')


def resolve_path(repo_root: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else (repo_root / path).resolve()


def changed_shell_files(repo_root: Path, merge_base: str) -> list[str]:
    result = git(repo_root, 'diff', '--name-only', '--diff-filter=AMR', merge_base, '--', 'scripts')
    return sorted(path for path in result.stdout.splitlines() if is_shell_path(repo_root, path.strip()))


def is_shell_path(repo_root: Path, rel_path: str) -> bool:
    if not rel_path.endswith('.sh'):
        return False
    path = repo_root / rel_path
    return path.is_file()


def evaluate(
    repo_root: Path,
    merge_base: str,
    touched: list[str],
    exceptions: set[str],
) -> dict[str, str]:
    offenders: list[str] = []
    exempted: list[str] = []
    checked: list[str] = []
    for rel_path in touched:
        if not is_shell_script(repo_root / rel_path):
            continue
        if rel_path in exceptions:
            exempted.append(rel_path)
            continue
        checked.append(rel_path)
        if not has_strict_mode(repo_root / rel_path):
            offenders.append(rel_path)
    return build_report(merge_base, touched, checked, exempted, offenders)


def is_shell_script(path: Path) -> bool:
    lines = read_script_lines(path)
    return bool(lines) and lines[0].startswith('#!') and ('bash' in lines[0] or 'sh' in lines[0])


def has_strict_mode(path: Path) -> bool:
    lines = read_script_lines(path)
    for line in lines[1:8]:
        if line.strip() == 'set -euo pipefail':
            return True
    return False


def read_script_lines(path: Path) -> list[str]:
    return path.read_text(encoding='utf-8', errors='ignore').splitlines()


def build_report(
    merge_base: str,
    touched: list[str],
    checked: list[str],
    exempted: list[str],
    offenders: list[str],
) -> dict[str, str]:
    report = {
        'status': 'pass',
        'final_decision': 'GO',
        'reason_codes': 'none',
        'merge_base': merge_base,
        'touched_shell_scripts': csv(touched),
        'checked_shell_scripts': csv(checked),
        'exempted_shell_scripts': csv(exempted),
        'offending_shell_scripts': csv(offenders),
    }
    if offenders:
        report['status'] = 'fail'
        report['final_decision'] = 'NO-GO'
        report['reason_codes'] = 'touched_shell_strict_mode_missing_strict_mode'
        report['remediation'] = 'add set -euo pipefail near the top or land an explicit exception first'
    return report


def csv(values: list[str]) -> str:
    return ','.join(values) if values else 'none'


def emit_report(output_json: str, report: dict[str, str]) -> int:
    path = Path(output_json)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + '\n', encoding='utf-8')
    stream = sys.stdout if report['status'] == 'pass' else sys.stderr
    for key in sorted(report):
        print(f'{key}={report[key]}', file=stream)
    return 0 if report['status'] == 'pass' else 1


def git(repo_root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(['git', '-C', str(repo_root), *args], capture_output=True, check=False, text=True)


class PolicyError(Exception):
    def __init__(self, reason_code: str, message: str) -> None:
        super().__init__(message)
        self.reason_code = reason_code
        self.message = message

    def emit(self, output_json: str) -> int:
        report = {
            'status': 'fail',
            'final_decision': 'NO-GO',
            'reason_codes': self.reason_code,
            'message': self.message,
        }
        return emit_report(output_json, report)


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
