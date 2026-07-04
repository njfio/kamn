#!/usr/bin/env python3
"""Baseline parsing helpers for touched Rust size-policy checks."""

from __future__ import annotations

from typing import Any


def parse_oversized_file_baseline(entries: list[Any]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for entry in entries:
        path, line_count = parse_oversized_file_entry(entry)
        if path in counts:
            raise ValueError(f'duplicate oversized file baseline entry: {path}')
        counts[path] = line_count
    return counts


def parse_oversized_file_entry(entry: Any) -> tuple[str, int]:
    if not isinstance(entry, dict):
        raise ValueError('oversized file entries must be objects')
    path = entry.get('path')
    line_count = entry.get('line_count')
    if not valid_rust_path(path):
        raise ValueError('oversized file path must be a relative Rust path')
    if not positive_int(line_count):
        raise ValueError('oversized file line_count must be a positive integer')
    return path, line_count


def parse_oversized_function_baseline(entries: list[Any]) -> dict[tuple[str, str], int]:
    counts: dict[tuple[str, str], int] = {}
    for entry in entries:
        path, header_key, line_count = parse_oversized_function_entry(entry)
        key = (path, header_key)
        if key in counts:
            raise ValueError(f'duplicate oversized function baseline entry: {path}::{header_key}')
        counts[key] = line_count
    return counts


def parse_oversized_function_entry(entry: Any) -> tuple[str, str, int]:
    if not isinstance(entry, dict):
        raise ValueError('oversized function entries must be objects')
    path = entry.get('path')
    header_key = entry.get('header_key')
    line_count = entry.get('line_count')
    if not valid_rust_path(path):
        raise ValueError('oversized function path must be a relative Rust path')
    if not isinstance(header_key, str) or not header_key:
        raise ValueError('oversized function header_key must be a non-empty string')
    if not positive_int(line_count):
        raise ValueError('oversized function line_count must be a positive integer')
    return path, header_key, line_count


def valid_rust_path(path: Any) -> bool:
    return isinstance(path, str) and path.endswith('.rs') and not path.startswith('/')


def positive_int(value: Any) -> bool:
    return isinstance(value, int) and value > 0
