#!/usr/bin/env python3
"""Support helpers for touched Rust size-policy checks."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from scripts.ci.touched_rust_function_scan import extract_function_spans

THRESHOLD_SCHEMA = 'kamn.ci.touched-rust-size-policy-thresholds.v1'
BASELINE_SCHEMA = 'kamn.ci.touched-rust-size-policy-baseline.v1'


def validate_threshold_payload(payload: dict[str, Any]) -> tuple[int, int]:
    if payload.get('schema_version') != THRESHOLD_SCHEMA:
        raise ValueError('threshold schema mismatch')
    file_lines = payload.get('max_file_lines')
    function_lines = payload.get('max_function_lines')
    if not isinstance(file_lines, int) or file_lines <= 0:
        raise ValueError('max_file_lines must be a positive integer')
    if not isinstance(function_lines, int) or function_lines <= 0:
        raise ValueError('max_function_lines must be a positive integer')
    return file_lines, function_lines


def validate_baseline_payload(payload: dict[str, Any]) -> None:
    if payload.get('schema_version') != BASELINE_SCHEMA:
        raise ValueError('baseline schema mismatch')
    _require_list(payload, 'oversized_files')
    _require_list(payload, 'oversized_functions')


def _require_list(payload: dict[str, Any], key: str) -> None:
    if not isinstance(payload.get(key), list):
        raise ValueError(f'{key} must be a list')


def rust_source_paths(repo_root: Path) -> list[str]:
    paths = []
    for path in repo_root.glob('crates/**/*.rs'):
        if path.is_file():
            paths.append(path.relative_to(repo_root).as_posix())
    paths.sort()
    return paths


def build_baseline(repo_root: Path, max_file_lines: int, max_function_lines: int) -> dict[str, Any]:
    oversized_files: list[dict[str, Any]] = []
    oversized_functions: list[dict[str, Any]] = []
    for rel_path in rust_source_paths(repo_root):
        raw = (repo_root / rel_path).read_text(encoding='utf-8')
        _append_oversized_file(oversized_files, rel_path, raw, max_file_lines)
        _append_oversized_functions(
            oversized_functions,
            rel_path,
            raw,
            max_function_lines,
        )
    return {
        'schema_version': BASELINE_SCHEMA,
        'captured_at': '2026-03-09',
        'max_file_lines': max_file_lines,
        'max_function_lines': max_function_lines,
        'oversized_files': oversized_files,
        'oversized_functions': oversized_functions,
    }


def _append_oversized_file(
    oversized_files: list[dict[str, Any]],
    rel_path: str,
    raw: str,
    max_file_lines: int,
) -> None:
    line_count = len(raw.splitlines())
    if line_count > max_file_lines:
        oversized_files.append({'path': rel_path, 'line_count': line_count})


def _append_oversized_functions(
    oversized_functions: list[dict[str, Any]],
    rel_path: str,
    raw: str,
    max_function_lines: int,
) -> None:
    for span in extract_function_spans(rel_path, raw):
        if span.line_count > max_function_lines:
            oversized_functions.append(span.baseline_entry())
