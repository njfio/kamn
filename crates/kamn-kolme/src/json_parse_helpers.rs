//! Shared low-level JSON token scanning helpers used by Kolme parser policies.

/// Splits one string by `delimiter`, ignoring delimiters inside quoted segments.
pub(crate) fn split_unquoted(input: &str, delimiter: char) -> Result<Vec<String>, &'static str> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && in_quotes {
            current.push(ch);
            escape = true;
            continue;
        }

        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            continue;
        }

        if ch == delimiter && !in_quotes {
            if current.trim().is_empty() {
                return Err("empty segment");
            }
            parts.push(current.trim().to_owned());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if in_quotes {
        return Err("unterminated quoted string");
    }
    if current.trim().is_empty() {
        return Err("empty trailing segment");
    }
    parts.push(current.trim().to_owned());
    Ok(parts)
}

/// Advances `cursor` over ASCII whitespace bytes.
pub(crate) fn skip_ascii_whitespace(value: &str, mut cursor: usize) -> usize {
    while let Some(byte) = value.as_bytes().get(cursor).copied() {
        if byte.is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        break;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::{skip_ascii_whitespace, split_unquoted};

    #[test]
    fn spec_c03_split_unquoted_keeps_quoted_delimiters() {
        let parts = split_unquoted(r#""a,b",c"#, ',').expect("split should pass");
        assert_eq!(parts, vec![r#""a,b""#.to_owned(), "c".to_owned()]);
    }

    #[test]
    fn spec_c03_split_unquoted_handles_escaped_quotes() {
        let parts = split_unquoted(r#""a\"b",c"#, ',').expect("split should pass");
        assert_eq!(parts, vec![r#""a\"b""#.to_owned(), "c".to_owned()]);
    }

    #[test]
    fn unit_skip_ascii_whitespace_advances_cursor() {
        assert_eq!(skip_ascii_whitespace("  \n\tvalue", 0), 4);
    }
}
