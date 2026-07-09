use sha2::{Digest, Sha256};

pub(crate) fn digest_field(raw: &str, field: &str) -> String {
    let marker = format!("\"{field}\":\"");
    let start = raw
        .find(marker.as_str())
        .expect("digest field should exist")
        + marker.len();
    let end = raw[start..].find('"').expect("digest field should end");
    raw[start..start + end].to_owned()
}

pub(crate) fn refresh_digest(raw: String) -> String {
    let digest = tagged_sha256(without_string_field(raw.as_str(), "view_digest").as_str());
    replace_string_field(raw.as_str(), "view_digest", digest.as_str())
}

pub(crate) fn with_digest(raw: String, field: &str) -> String {
    let digest = tagged_sha256(without_string_field(raw.as_str(), field).as_str());
    replace_string_field(raw.as_str(), field, digest.as_str())
}

fn replace_string_field(raw: &str, field: &str, value: &str) -> String {
    let marker = format!("\"{field}\":\"");
    let start = raw
        .find(marker.as_str())
        .expect("digest field should exist")
        + marker.len();
    let end = raw[start..].find('"').expect("digest field should end");
    format!("{}{}{}", &raw[..start], value, &raw[start + end..])
}

fn without_string_field(raw: &str, field: &str) -> String {
    let marker = format!("\"{field}\":\"");
    let start = raw
        .find(marker.as_str())
        .expect("digest field should exist");
    let value_start = start + marker.len();
    let end = value_start
        + raw[value_start..]
            .find('"')
            .expect("digest field should end")
        + 1;
    remove_pair(raw, start, end)
}

fn remove_pair(raw: &str, start: usize, end: usize) -> String {
    if raw[end..].starts_with(',') {
        return format!("{}{}", &raw[..start], &raw[end + 1..]);
    }
    format!("{}{}", &raw[..start - 1], &raw[end..])
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
