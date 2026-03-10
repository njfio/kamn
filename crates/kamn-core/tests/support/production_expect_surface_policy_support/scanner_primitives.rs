use super::CodeScanState;

pub fn starts_with(bytes: &[u8], index: usize, needle: &[u8]) -> bool {
    bytes
        .get(index..index + needle.len())
        .is_some_and(|candidate| candidate == needle)
}

pub fn raw_string_start(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let prefix_len = match bytes.get(index..index + 2) {
        Some(b"br") | Some(b"rb") => 2,
        _ if bytes.get(index) == Some(&b'r') => 1,
        _ => return None,
    };
    let mut cursor = index + prefix_len;
    let mut hash_count = 0_usize;
    while bytes.get(cursor) == Some(&b'#') {
        hash_count += 1;
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((prefix_len + hash_count + 1, hash_count))
}

pub fn char_literal_end(bytes: &[u8], index: usize) -> Option<usize> {
    if bytes.get(index) != Some(&b'\'') {
        return None;
    }
    let mut cursor = index + 1;
    let mut escaped = false;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        if escaped {
            escaped = false;
        } else if bytes[cursor] == b'\\' {
            escaped = true;
        } else if bytes[cursor] == b'\'' {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

pub fn closes_raw_string(bytes: &[u8], index: usize, hash_count: usize) -> bool {
    if bytes.get(index) != Some(&b'"') {
        return false;
    }
    (0..hash_count).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'))
}

pub fn consume_raw_string(
    bytes: &[u8],
    index: usize,
    state: &mut CodeScanState,
    hash_count: usize,
    closes: fn(&[u8], usize, usize) -> bool,
) -> usize {
    if closes(bytes, index, hash_count) {
        state.raw_string_hash_count = None;
        return index + hash_count + 2;
    }
    index + 1
}

pub fn consume_block_comment(
    bytes: &[u8],
    index: usize,
    state: &mut CodeScanState,
    starts: fn(&[u8], usize, &[u8]) -> bool,
) -> usize {
    if starts(bytes, index, b"/*") {
        state.block_comment_depth += 1;
        return index + 2;
    }
    if starts(bytes, index, b"*/") {
        state.block_comment_depth -= 1;
        return index + 2;
    }
    index + 1
}

pub fn consume_string(bytes: &[u8], index: usize, state: &mut CodeScanState) -> usize {
    if state.escaped {
        state.escaped = false;
    } else if bytes[index] == b'\\' {
        state.escaped = true;
    } else if bytes[index] == b'"' {
        state.in_string = false;
    }
    index + 1
}

pub fn skip_line_comment(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        cursor += 1;
    }
    cursor
}
