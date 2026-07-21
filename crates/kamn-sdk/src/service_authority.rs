use super::{json_string_field, SdkError};

pub(super) fn receipt_fields(body: &str) -> Result<(String, String), SdkError> {
    let receipt_id = json_string_field(body, "receipt_id")?;
    if receipt_id.is_empty() {
        return Err(invalid("service receipt id was empty"));
    }
    let digest = json_string_field(body, "receipt_digest")?;
    validate_digest(digest.as_str())?;
    Ok((receipt_id, digest))
}

pub(super) fn profile_commitment(body: &str) -> Result<String, SdkError> {
    let commitment = json_string_field(body, "profile_commitment")?;
    validate_digest(commitment.as_str())?;
    Ok(commitment)
}

fn validate_digest(value: &str) -> Result<(), SdkError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(invalid("service authority digest was malformed"));
    };
    if hex.len() == 64 && hex.bytes().all(is_lower_hex) {
        return Ok(());
    }
    Err(invalid("service authority digest was malformed"))
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn invalid(message: &'static str) -> SdkError {
    SdkError::TransportFailure(message)
}
