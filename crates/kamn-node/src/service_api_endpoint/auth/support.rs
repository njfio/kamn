use super::*;

pub(super) const SELF_CERTIFYING_AGENT_DID_KEY_PREFIX: &str = "kamn:did:agent:pkh-";

pub(crate) fn header_value<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<&'a str> {
    headers.get(name).map(String::as_str)
}

pub(crate) fn service_api_signature_state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}

pub(super) fn normalized_public_key_hexes_match(left: &str, right: &str) -> bool {
    let normalized_left = ascii_lowercase_bytes(left.as_bytes());
    let normalized_right = ascii_lowercase_bytes(right.as_bytes());
    constant_time_eq_bytes(normalized_left.as_slice(), normalized_right.as_slice())
}

fn ascii_lowercase_bytes(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

fn constant_time_eq_bytes(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}
