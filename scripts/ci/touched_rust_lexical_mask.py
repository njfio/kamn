#!/usr/bin/env python3
"""Mask non-code Rust tokens so brace counting can stay lexical and deterministic."""

from __future__ import annotations


def strip_non_code(text: str) -> str:
    out: list[str] = []
    index = 0
    state = {'block_depth': 0, 'in_string': False, 'raw_hashes': -1}
    while index < len(text):
        index = _step(text, index, out, state)
    return ''.join(out)


def _step(text: str, index: int, out: list[str], state: dict[str, int | bool]) -> int:
    if state['block_depth']:
        return _consume_block_comment(text, index, out, state)
    if state['in_string']:
        return _consume_string(text, index, out, state)
    return _consume_code(text, index, out, state)


def _consume_block_comment(
    text: str,
    index: int,
    out: list[str],
    state: dict[str, int | bool],
) -> int:
    duo = text[index : index + 2]
    if duo == '/*':
        state['block_depth'] += 1
        out.extend('  ')
        return index + 2
    if duo == '*/':
        state['block_depth'] -= 1
        out.extend('  ')
        return index + 2
    out.append('\n' if text[index] == '\n' else ' ')
    return index + 1


def _consume_string(
    text: str,
    index: int,
    out: list[str],
    state: dict[str, int | bool],
) -> int:
    char = text[index]
    raw_hashes = int(state['raw_hashes'])
    if char == '\\' and raw_hashes < 0 and index + 1 < len(text):
        out.extend('  ')
        return index + 2
    if _is_raw_string_end(text, index, raw_hashes):
        out.append(' ')
        out.extend(' ' * raw_hashes)
        state['in_string'] = False
        state['raw_hashes'] = -1
        return index + raw_hashes + 1
    if raw_hashes < 0 and char == '"':
        out.append(' ')
        state['in_string'] = False
        return index + 1
    out.append('\n' if char == '\n' else ' ')
    return index + 1


def _consume_code(
    text: str,
    index: int,
    out: list[str],
    state: dict[str, int | bool],
) -> int:
    duo = text[index : index + 2]
    if duo == '//':
        return _consume_line_comment(text, index, out)
    if duo == '/*':
        state['block_depth'] = 1
        out.extend('  ')
        return index + 2
    raw_hashes = _raw_hash_count(text, index)
    if raw_hashes >= 0:
        out.extend(' ' * (raw_hashes + 2))
        state['in_string'] = True
        state['raw_hashes'] = raw_hashes
        return index + raw_hashes + 2
    if text[index] == '"':
        out.append(' ')
        state['in_string'] = True
        state['raw_hashes'] = -1
        return index + 1
    char_length = _char_literal_length(text, index)
    if char_length:
        out.extend(' ' * char_length)
        return index + char_length
    out.append(text[index])
    return index + 1


def _consume_line_comment(text: str, index: int, out: list[str]) -> int:
    while index < len(text) and text[index] != '\n':
        out.append(' ')
        index += 1
    return index


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
        if text[cursor] == '\n':
            return 0
        if text[cursor] == "'" and text[cursor - 1] != '\\':
            return cursor - index + 1
        cursor += 1
    return 0


def _is_raw_string_end(text: str, index: int, raw_hashes: int) -> bool:
    if raw_hashes < 0 or text[index] != '"':
        return False
    return text[index + 1 : index + 1 + raw_hashes] == '#' * raw_hashes
