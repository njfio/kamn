mod scanner_primitives;

use scanner_primitives::{
    char_literal_end, closes_raw_string, consume_block_comment, consume_raw_string,
    consume_string, raw_string_start, skip_line_comment, starts_with,
};

const EXPECT_CALL_TOKEN: &[u8] = b".expect(";

#[derive(Debug, Default, Clone)]
struct CodeScanState {
    block_comment_depth: usize,
    raw_string_hash_count: Option<usize>,
    in_string: bool,
    escaped: bool,
}

pub fn count_expect_occurrences_excluding_cfg_test(raw: &str) -> i64 {
    let bytes = raw.as_bytes();
    let mut count = 0_i64;
    let mut index = 0_usize;
    let mut state = CodeScanState::default();

    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, &mut state) {
            index = next;
        } else if starts_with(bytes, index, b"#[cfg(test)]") {
            index = skip_cfg_test_item(bytes, index);
        } else if starts_with(bytes, index, EXPECT_CALL_TOKEN) {
            count += 1;
            index += EXPECT_CALL_TOKEN.len();
        } else {
            index += 1;
        }
    }

    count
}

fn consume_non_code(bytes: &[u8], index: usize, state: &mut CodeScanState) -> Option<usize> {
    consume_active_non_code(bytes, index, state).or_else(|| start_non_code(bytes, index, state))
}

fn consume_active_non_code(
    bytes: &[u8],
    index: usize,
    state: &mut CodeScanState,
) -> Option<usize> {
    if let Some(hash_count) = state.raw_string_hash_count {
        return Some(consume_raw_string(bytes, index, state, hash_count, closes_raw_string));
    }
    if state.block_comment_depth > 0 {
        return Some(consume_block_comment(bytes, index, state, starts_with));
    }
    state.in_string.then(|| consume_string(bytes, index, state))
}

fn start_non_code(bytes: &[u8], index: usize, state: &mut CodeScanState) -> Option<usize> {
    if starts_with(bytes, index, b"//") {
        return Some(skip_line_comment(bytes, index + 2));
    }
    if starts_with(bytes, index, b"/*") {
        state.block_comment_depth = 1;
        return Some(index + 2);
    }
    if let Some((prefix_len, hash_count)) = raw_string_start(bytes, index) {
        state.raw_string_hash_count = Some(hash_count);
        return Some(index + prefix_len);
    }
    if let Some(end) = char_literal_end(bytes, index) {
        return Some(end);
    }
    if starts_with(bytes, index, b"b\"") {
        state.in_string = true;
        state.escaped = false;
        return Some(index + 2);
    }
    (bytes[index] == b'"').then(|| {
        state.in_string = true;
        state.escaped = false;
        index + 1
    })
}

fn skip_whitespace(bytes: &[u8], mut index: usize, state: &mut CodeScanState) -> usize {
    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, state) {
            index = next;
        } else if bytes[index].is_ascii_whitespace() {
            index += 1;
        } else {
            return index;
        }
    }
    index
}

fn skip_attribute(bytes: &[u8], mut index: usize, state: &mut CodeScanState) -> usize {
    let mut depth = 0_i64;
    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, state) {
            index = next;
        } else if bytes[index] == b'[' {
            depth += 1;
            index += 1;
        } else if bytes[index] == b']' {
            depth -= 1;
            index += 1;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}

fn skip_cfg_test_item(bytes: &[u8], mut index: usize) -> usize {
    let mut state = CodeScanState::default();
    index += "#[cfg(test)]".len();
    index = skip_whitespace(bytes, index, &mut state);
    while starts_with(bytes, index, b"#[") {
        index = skip_attribute(bytes, index + 1, &mut state);
        index = skip_whitespace(bytes, index, &mut state);
    }
    skip_cfg_test_body(bytes, index, &mut state)
}

fn skip_cfg_test_body(bytes: &[u8], mut index: usize, state: &mut CodeScanState) -> usize {
    let mut body_depth = 0_i64;
    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, state) {
            index = next;
        } else if body_depth == 0 && bytes[index] == b';' {
            return index + 1;
        } else if bytes[index] == b'{' {
            body_depth += 1;
            index += 1;
        } else if bytes[index] == b'}' {
            body_depth -= 1;
            index += 1;
            if body_depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    bytes.len()
}
