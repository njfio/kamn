pub(super) struct ClaimView<'a> {
    pub(super) raw: &'a str,
    pub(super) id: String,
    pub(super) label: String,
    pub(super) required: bool,
    pub(super) status: String,
}

pub(super) fn validate_json_delimiters(report_json: &str) -> Result<(), String> {
    let trimmed = report_json.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return Err("malformed MVP demo report JSON".to_owned());
    }
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in report_json.chars() {
        if in_string {
            update_string_state(character, &mut in_string, &mut escaped);
            continue;
        }
        update_delimiter_stack(&mut stack, character, &mut in_string)?;
    }
    if in_string || !stack.is_empty() {
        return Err("malformed MVP demo report JSON".to_owned());
    }
    Ok(())
}

pub(super) fn parse_claims(report_json: &str) -> Result<Vec<ClaimView<'_>>, String> {
    let section = claim_matrix_section(report_json)?;
    if section.trim().is_empty() {
        return Err("MVP claim_matrix is empty".to_owned());
    }
    section
        .split("},{")
        .map(parse_claim)
        .collect::<Result<Vec<_>, _>>()
}

pub(super) fn require_marker(haystack: &str, marker: &str, context: &str) -> Result<(), String> {
    if haystack.contains(marker) {
        return Ok(());
    }
    Err(format!(
        "missing MVP demo report marker for {context}: {marker}"
    ))
}

fn update_string_state(character: char, in_string: &mut bool, escaped: &mut bool) {
    if *escaped {
        *escaped = false;
    } else if character == '\\' {
        *escaped = true;
    } else if character == '"' {
        *in_string = false;
    }
}

fn update_delimiter_stack(
    stack: &mut Vec<char>,
    character: char,
    in_string: &mut bool,
) -> Result<(), String> {
    match character {
        '"' => *in_string = true,
        '{' | '[' => stack.push(character),
        '}' => pop_expected(stack, '{')?,
        ']' => pop_expected(stack, '[')?,
        _ => {}
    }
    Ok(())
}

fn pop_expected(stack: &mut Vec<char>, expected: char) -> Result<(), String> {
    if stack.pop() == Some(expected) {
        return Ok(());
    }
    Err("malformed MVP demo report JSON".to_owned())
}

fn parse_claim(raw: &str) -> Result<ClaimView<'_>, String> {
    Ok(ClaimView {
        raw,
        id: extract_string(raw, "id")?,
        label: extract_string(raw, "label")?,
        required: extract_bool(raw, "required")?,
        status: extract_string(raw, "status")?,
    })
}

fn claim_matrix_section(report_json: &str) -> Result<&str, String> {
    let start = report_json
        .find("\"claim_matrix\":[")
        .ok_or_else(|| "missing claim_matrix".to_owned())?;
    let content_start = start + "\"claim_matrix\":[".len();
    let rest = &report_json[content_start..];
    let end = rest
        .find("],\"no_go\"")
        .ok_or_else(|| "missing no_go after claim_matrix".to_owned())?;
    Ok(&rest[..end])
}

pub(super) fn extract_string(raw: &str, field: &str) -> Result<String, String> {
    extract_optional_string(raw, field).ok_or_else(|| format!("missing claim field: {field}"))
}

pub(super) fn extract_optional_string(raw: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\":\"");
    let start = raw.find(marker.as_str())?;
    let value_start = start + marker.len();
    let value = &raw[value_start..];
    let end = value.find('"')?;
    Some(value[..end].to_owned())
}

pub(super) fn extract_bool(raw: &str, field: &str) -> Result<bool, String> {
    let marker = format!("\"{field}\":");
    let start = raw
        .find(marker.as_str())
        .ok_or_else(|| format!("missing claim field: {field}"))?;
    let value = &raw[start + marker.len()..];
    Ok(value.starts_with("true"))
}

pub(super) fn extract_u64(raw: &str, field: &str) -> Result<u64, String> {
    let marker = format!("\"{field}\":");
    let start = raw
        .find(marker.as_str())
        .ok_or_else(|| format!("missing claim field: {field}"))?;
    let digits = raw[start + marker.len()..]
        .chars()
        .skip_while(|item| item.is_ascii_whitespace())
        .take_while(|item| item.is_ascii_digit())
        .collect::<String>();
    digits
        .parse::<u64>()
        .map_err(|error| format!("invalid claim field {field}: {error}"))
}
