use crate::SdkError;

pub(super) fn set_once(
    slot: &mut Option<String>,
    value: &str,
    field: &'static str,
    key: &'static str,
) -> Result<(), SdkError> {
    if slot.is_some() {
        return Err(SdkError::InvalidInput {
            field,
            reason: duplicate_key_reason(key),
        });
    }
    *slot = Some(value.to_owned());
    Ok(())
}

pub(super) fn set_nonce(
    slot: &mut Option<u64>,
    value: &str,
    field: &'static str,
) -> Result<(), SdkError> {
    if slot.is_some() {
        return Err(SdkError::InvalidInput {
            field: "handshake_frame",
            reason: "duplicate key: nonce",
        });
    }
    *slot = Some(value.parse::<u64>().map_err(|_| SdkError::InvalidInput {
        field,
        reason: "must be an unsigned integer",
    })?);
    Ok(())
}

pub(super) fn required_string(
    value: Option<String>,
    field: &'static str,
) -> Result<String, SdkError> {
    value.ok_or(SdkError::InvalidInput {
        field,
        reason: "missing required key",
    })
}

pub(super) fn required_nonce(value: Option<u64>, field: &'static str) -> Result<u64, SdkError> {
    value.ok_or(SdkError::InvalidInput {
        field,
        reason: "missing required key",
    })
}

pub(super) fn require_exact(
    actual: String,
    field: &'static str,
    expected: &'static str,
    reason: &'static str,
) -> Result<(), SdkError> {
    if actual == expected {
        Ok(())
    } else {
        Err(SdkError::InvalidInput { field, reason })
    }
}

pub(super) fn verify_field_match(
    matches: bool,
    field: &'static str,
    reason: &'static str,
) -> Result<(), SdkError> {
    if matches {
        Ok(())
    } else {
        Err(SdkError::InvalidInput { field, reason })
    }
}

fn duplicate_key_reason(key: &str) -> &'static str {
    match key {
        "frame" => "duplicate key: frame",
        "version" => "duplicate key: version",
        "profile" => "duplicate key: profile",
        "from" => "duplicate key: from",
        "to" => "duplicate key: to",
        "signer_public_key" => "duplicate key: signer_public_key",
        "signature" => "duplicate key: signature",
        _ => "duplicate key",
    }
}
