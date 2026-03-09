#!/usr/bin/env python3
"""Lexical Rust function scanning helpers for CI policy checks."""

from __future__ import annotations

from dataclasses import dataclass
import re

from scripts.ci.touched_rust_lexical_mask import strip_non_code

FN_HEADER_RE = re.compile(
    r'^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?'
    r'(?:unsafe\s+)?(?:extern\s+"[^"]+"\s+)?fn\s+'
    r'(?P<name>[A-Za-z_][A-Za-z0-9_]*)\b'
)


@dataclass(frozen=True)
class FunctionSpan:
    path: str
    name: str
    start_line: int
    end_line: int
    line_count: int
    header_key: str

    def baseline_entry(self) -> dict[str, object]:
        return {
            'path': self.path,
            'name': self.name,
            'start_line': self.start_line,
            'end_line': self.end_line,
            'line_count': self.line_count,
            'header_key': self.header_key,
        }

    def offender_id(self) -> str:
        return f'{self.path}::{self.name}@{self.start_line}'


def extract_function_spans(path: str, text: str) -> list[FunctionSpan]:
    lines = strip_non_code(text).splitlines()
    counts: dict[str, int] = {}
    spans: list[FunctionSpan] = []
    index = 0
    while index < len(lines):
        if not FN_HEADER_RE.match(lines[index]):
            index += 1
            continue
        header = _header_result(lines, index)
        if header is None:
            index += 1
            continue
        spans.append(_build_span(path, lines, index, header, counts))
        index = spans[-1].end_line
    return spans


def _build_span(
    path: str,
    lines: list[str],
    start: int,
    header: tuple[str, str, int],
    counts: dict[str, int],
) -> FunctionSpan:
    header_key, name, header_end = header
    final_line = _find_end_line(lines, start, header_end, path)
    counts[header_key] = counts.get(header_key, 0) + 1
    return FunctionSpan(
        path=path,
        name=name,
        start_line=start + 1,
        end_line=final_line + 1,
        line_count=final_line - start + 1,
        header_key=f'{_normalize_header(header_key)}#{counts[header_key]}',
    )


def _header_result(lines: list[str], start: int) -> tuple[str, str, int] | None:
    header_lines: list[str] = []
    index = start
    while index < len(lines):
        header_lines.append(lines[index].strip())
        header_text = ' '.join(part for part in header_lines if part)
        brace_index = header_text.find('{')
        semi_index = header_text.find(';')
        if semi_index >= 0 and (brace_index < 0 or semi_index < brace_index):
            return None
        if brace_index >= 0:
            return _parsed_header(header_text, brace_index, index)
        index += 1
    raise ValueError('function header missing opening brace')


def _parsed_header(header_text: str, brace_index: int, index: int) -> tuple[str, str, int] | None:
    match = FN_HEADER_RE.match(header_text)
    if not match:
        return None
    return header_text[:brace_index], match.group('name'), index


def _find_end_line(lines: list[str], start: int, header_end: int, path: str) -> int:
    depth = 0
    index = start
    while index < len(lines):
        depth += lines[index].count('{') - lines[index].count('}')
        if depth == 0 and index >= header_end:
            return index
        index += 1
    raise ValueError(f'unbalanced braces while parsing {path}:{start + 1}')


def _normalize_header(header: str) -> str:
    return ' '.join(header.split())
