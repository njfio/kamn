use sha2::{Digest, Sha256};

pub(crate) struct ArtifactJson {
    pub(crate) json: String,
    pub(crate) digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreeAgentViewDigests {
    pub(crate) agent_a: String,
    pub(crate) agent_b: String,
    pub(crate) agent_c_verifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreeAgentReceiptDigests {
    pub(crate) agent_a: String,
    pub(crate) agent_b: String,
    pub(crate) agent_c_verifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ThreeAgentArtifactDigests {
    pub(crate) transcript: String,
    pub(crate) views: ThreeAgentViewDigests,
    pub(crate) receipts: ThreeAgentReceiptDigests,
}

pub(crate) fn attach_json_digest(raw: String, field: &str) -> Result<ArtifactJson, String> {
    let digest = digest_json_without_field(raw.as_str(), field)?;
    let placeholder = format!("\"{field}\":\"\"");
    let replacement = format!("\"{field}\":\"{digest}\"");
    if !raw.contains(placeholder.as_str()) {
        return Err(format!("missing digest placeholder field: {field}"));
    }
    Ok(ArtifactJson {
        json: raw.replace(placeholder.as_str(), replacement.as_str()),
        digest,
    })
}

pub(crate) fn validate_json_digest(
    raw: &str,
    field: &str,
    expected: &str,
    context: &str,
) -> Result<(), String> {
    let actual = digest_json_without_field(raw, field)?;
    if actual == expected {
        return Ok(());
    }
    Err(format!("{context} digest mismatch"))
}

pub(crate) fn digest_json_without_field(raw: &str, field: &str) -> Result<String, String> {
    Ok(tagged_sha256(
        json_without_string_field(raw, field)?.as_str(),
    ))
}

fn tagged_sha256(value: &str) -> String {
    format!("sha256:{}", sha256_hex(value))
}

fn sha256_hex(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(value.as_bytes());
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn json_without_string_field(raw: &str, field: &str) -> Result<String, String> {
    let (field_start, pair_end) = string_field_span(raw, field)?;
    let (remove_start, remove_end) = removal_span(raw, field_start, pair_end)?;
    let mut output = String::with_capacity(raw.len().saturating_sub(remove_end - remove_start));
    output.push_str(&raw[..remove_start]);
    output.push_str(&raw[remove_end..]);
    Ok(output)
}

fn string_field_span(raw: &str, field: &str) -> Result<(usize, usize), String> {
    let marker = format!("\"{field}\":\"");
    let field_start = raw
        .find(marker.as_str())
        .ok_or_else(|| format!("missing digest field: {field}"))?;
    let value_start = field_start + marker.len();
    let value_end = closing_quote(raw, value_start)?;
    Ok((field_start, value_end + 1))
}

fn closing_quote(raw: &str, start: usize) -> Result<usize, String> {
    let mut escaped = false;
    for (offset, character) in raw[start..].char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Ok(start + offset);
        }
    }
    Err("unterminated digest string field".to_owned())
}

fn removal_span(raw: &str, field_start: usize, pair_end: usize) -> Result<(usize, usize), String> {
    if raw[pair_end..].starts_with(',') {
        return Ok((field_start, pair_end + 1));
    }
    if field_start > 0 && raw.as_bytes()[field_start - 1] == b',' {
        return Ok((field_start - 1, pair_end));
    }
    Err("digest field must be in a JSON object with a sibling field".to_owned())
}
