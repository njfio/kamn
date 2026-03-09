#!/usr/bin/env python3
"""Lexical Rust function scanning helpers for CI policy checks."""

from __future__ import annotations

from dataclasses import dataclass
import re

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


def normalize_header(header: str) -> str:
    return ' '.join(header.split())


def _raw_hash_count(text: str, index: int) -> int:
    if index >= len(text) or text[index] != 'r':
        return -1
    cursor = index + 1
    while cursor < len(text) and text[cursor] == '#':
        cursor += 1
    if cursor < len(text) and text[cursor] == '"':
        return cursor - index - 1
    return -1


def _char_literal_length(text: str, index: int) -> int:
    if text[index] != "'":
        return 0
    cursor = index + 1
    while cursor < len(text) and cursor - index <= 6:
        if text[cursor] == "\n":
            return 0
        if text[cursor] == "'" and text[cursor - 1] != "\\":
            return cursor - index + 1
        cursor += 1
    return 0


def strip_non_code(text: str) -> str:
    out: list[str] = []
    index = 0
    block_depth = 0
    in_string = False
    raw_hashes = -1
    while index < len(text):
        char = text[index]
        duo = text[index : index + 2]
        if block_depth:
            if duo == '/*':
                block_depth += 1
                out.extend('  ')
                index += 2
                continue
            if duo == '*/':
                block_depth -= 1
                out.extend('  ')
                index += 2
                continue
            out.append('\n' if char == '\n' else ' ')
            index += 1
            continue
        if in_string:
            if char == '\\' and raw_hashes < 0 and index + 1 < len(text):
                out.extend('  ')
                index += 2
                continue
            if _is_raw_string_end(text, index, raw_hashes):
                out.append(' ')
                out.extend(' ' * raw_hashes)
                index += raw_hashes + 1
                in_string = False
                raw_hashes = -1
                continue
            if raw_hashes < 0 and char == '"':
                out.append(' ')
                index += 1
                in_string = False
                continue
            out.append('\n' if char == '\n' else ' ')
            index += 1
            continue
        if duo == '//':
            index = _consume_line_comment(text, index, out)
            continue
        if duo == '/*':
            block_depth = 1
            out.extend('  ')
            index += 2
            continue
        raw_hash_count = _raw_hash_count(text, index)
        if raw_hash_count >= 0:
            out.extend(' ' * (raw_hash_count + 2))
            index += raw_hash_count + 2
            in_string = True
            raw_hashes = raw_hash_count
            continue
        if char == '"':
            out.append(' ')
            index += 1
            in_string = True
            raw_hashes = -1
            continue
        char_literal_length = _char_literal_length(text, index)
        if char_literal_length:
            out.extend(' ' * char_literal_length)
            index += char_literal_length
            continue
        out.append(char)
        index += 1
    return ''.join(out)


def _consume_line_comment(text: str, index: int, out: list[str]) -> int:
    while index < len(text) and text[index] != '\n':
        out.append(' ')
        index += 1
    return index


def _is_raw_string_end(text: str, index: int, raw_hashes: int) -> bool:
    if raw_hashes < 0 or text[index] != '"':
        return False
    return text[index + 1 : index + 1 + raw_hashes] == '#' * raw_hashes


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
            match = FN_HEADER_RE.match(header_text)
            if not match:
                return None
            return normalize_header(header_text[:brace_index]), match.group('name'), index
        index += 1
    raise ValueError('function header missing opening brace')


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
        header_key, name, end_line = header
        final_line = _find_end_line(lines, index, end_line, path)
        counts[header_key] = counts.get(header_key, 0) + 1
        spans.append(
            FunctionSpan(
                path=path,
                name=name,
                start_line=index + 1,
                end_line=final_line + 1,
                line_count=final_line - index + 1,
                header_key=f'{header_key}#{counts[header_key]}',
            )
        )
        index = final_line + 1
    return spans


def _find_end_line(lines: list[str], start: int, header_end: int, path: str) -> int:
    depth = 0
    index = start
    while index < len(lines):
        depth += lines[index].count('{') - lines[index].count('}')
        if depth == 0 and index >= header_end:
            return index
        index += 1
    raise ValueError(f'unbalanced braces while parsing {path}:{start + 1}')
