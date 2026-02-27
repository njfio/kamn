//! Shared JSON scalar parsing and encoding helpers for Kolme policy modules.

/// Parses one JSON string token (including surrounding quotes).
pub(crate) fn parse_json_string_token(token: &str) -> Result<String, &'static str> {
    let trimmed = token.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err("token must be a quoted string");
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let mut output = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }

        let escaped = chars.next().ok_or("unterminated escape sequence")?;
        match escaped {
            '"' => output.push('"'),
            '\\' => output.push('\\'),
            '/' => output.push('/'),
            'b' => output.push('\u{0008}'),
            'f' => output.push('\u{000C}'),
            'n' => output.push('\n'),
            'r' => output.push('\r'),
            't' => output.push('\t'),
            'u' => output.push(parse_unicode_scalar(&mut chars)?),
            _ => return Err("unsupported escape sequence"),
        }
    }

    Ok(output)
}

/// Percent-encodes one UTF-8 value for deterministic query/path usage.
pub(crate) fn percent_encode_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        let ch = byte as char;
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '~') {
            encoded.push(ch);
        } else {
            encoded.push('%');
            encoded.push_str(format!("{byte:02X}").as_str());
        }
    }
    encoded
}

/// Advances one byte cursor past ASCII whitespace in a UTF-8 payload.
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

/// Splits a delimited string while honoring quoted JSON string sections.
pub(crate) fn split_unquoted_segments(
    input: &str,
    delimiter: char,
) -> Result<Vec<String>, &'static str> {
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

fn parse_unicode_scalar(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<char, &'static str> {
    let leading = parse_hex_quad(chars)?;
    if (0xD800..=0xDBFF).contains(&leading) {
        if chars.next() != Some('\\') || chars.next() != Some('u') {
            return Err("invalid unicode escape sequence");
        }
        let trailing = parse_hex_quad(chars)?;
        if !(0xDC00..=0xDFFF).contains(&trailing) {
            return Err("invalid unicode escape sequence");
        }

        let codepoint = 0x1_0000 + (((leading as u32 - 0xD800) << 10) | (trailing as u32 - 0xDC00));
        return char::from_u32(codepoint).ok_or("invalid unicode escape sequence");
    }

    if (0xDC00..=0xDFFF).contains(&leading) {
        return Err("invalid unicode escape sequence");
    }

    char::from_u32(leading as u32).ok_or("invalid unicode escape sequence")
}

fn parse_hex_quad(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> Result<u16, &'static str> {
    let mut value = 0_u16;
    for _ in 0..4 {
        let digit = chars.next().ok_or("invalid unicode escape sequence")?;
        let nibble = digit
            .to_digit(16)
            .ok_or("invalid unicode escape sequence")? as u16;
        value = (value << 4) | nibble;
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_json_string_token, percent_encode_component, skip_ascii_whitespace,
        split_unquoted_segments,
    };

    #[test]
    fn unit_json_scalar_policy_parses_common_escape_sequences() {
        let parsed =
            parse_json_string_token(r#""line\nquote:\"slash:\\tab:\t""#).expect("valid token");
        assert_eq!(parsed, "line\nquote:\"slash:\\tab:\t");
    }

    #[test]
    fn unit_json_scalar_policy_parses_unicode_escape_sequences() {
        let parsed = parse_json_string_token(r#""\u03B1\u03B2""#).expect("valid unicode token");
        assert_eq!(parsed, "αβ");
    }

    #[test]
    fn unit_json_scalar_policy_parses_surrogate_pairs() {
        let parsed = parse_json_string_token(r#""\uD83D\uDE80""#).expect("valid surrogate pair");
        assert_eq!(parsed, "\u{1F680}");
    }

    #[test]
    fn unit_json_scalar_policy_rejects_malformed_unicode_escape_sequences() {
        assert_eq!(
            parse_json_string_token(r#""\u03G1""#),
            Err("invalid unicode escape sequence")
        );
        assert_eq!(
            parse_json_string_token(r#""\uD83D""#),
            Err("invalid unicode escape sequence")
        );
    }

    #[test]
    fn unit_json_scalar_policy_percent_encodes_reserved_bytes() {
        let encoded = percent_encode_component("did:example:alice?nonce=7");
        assert_eq!(encoded, "did%3Aexample%3Aalice%3Fnonce%3D7");
    }

    #[test]
    fn unit_json_scalar_policy_skips_ascii_whitespace_prefix() {
        let payload = " \n\tfield";
        let cursor = skip_ascii_whitespace(payload, 0);
        assert_eq!(cursor, 3);
        assert_eq!(payload.as_bytes()[cursor], b'f');
    }

    #[test]
    fn unit_json_scalar_policy_splits_unquoted_segments_with_quoted_delimiters() {
        let parts = split_unquoted_segments(r#""a,b",c,"d,e""#, ',').expect("parts");
        assert_eq!(parts, vec![r#""a,b""#, "c", r#""d,e""#]);
    }

    #[test]
    fn unit_json_scalar_policy_rejects_empty_or_unterminated_segments() {
        assert_eq!(split_unquoted_segments("a,,b", ','), Err("empty segment"));
        assert_eq!(
            split_unquoted_segments(r#""unterminated"#, ','),
            Err("unterminated quoted string")
        );
    }
}
