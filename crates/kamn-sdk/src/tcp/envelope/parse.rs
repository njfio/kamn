use super::TcpSignedEnvelope;
use crate::{AgentDid, SdkError};

pub(super) struct EnvelopeFields {
    from: Option<String>,
    to: Option<String>,
    nonce: Option<u64>,
    state_hash: Option<String>,
    body: Option<String>,
    signer_public_key: Option<String>,
    signature: Option<String>,
}

impl EnvelopeFields {
    fn new() -> Self {
        Self {
            from: None,
            to: None,
            nonce: None,
            state_hash: None,
            body: None,
            signer_public_key: None,
            signature: None,
        }
    }
}

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
            field: "wire_payload",
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

pub(super) fn parse_envelope_fields(payload: &str) -> Result<EnvelopeFields, SdkError> {
    let mut fields = EnvelopeFields::new();
    for raw_line in payload.lines() {
        parse_envelope_line(raw_line, &mut fields)?;
    }
    Ok(fields)
}

pub(super) fn build_envelope(fields: EnvelopeFields) -> Result<TcpSignedEnvelope, SdkError> {
    Ok(TcpSignedEnvelope {
        from: AgentDid::parse(required_string(fields.from, "from")?.as_str())?,
        to: AgentDid::parse(required_string(fields.to, "to")?.as_str())?,
        nonce: required_nonce(fields.nonce, "nonce")?,
        state_hash: required_string(fields.state_hash, "state_hash")?,
        body: required_string(fields.body, "body")?,
        signer_public_key: required_string(fields.signer_public_key, "signer_public_key")?,
        signature: required_string(fields.signature, "signature")?,
    })
}

fn duplicate_key_reason(key: &str) -> &'static str {
    match key {
        "from" => "duplicate key: from",
        "to" => "duplicate key: to",
        "state_hash" => "duplicate key: state_hash",
        "body" => "duplicate key: body",
        "signer_public_key" => "duplicate key: signer_public_key",
        "signature" => "duplicate key: signature",
        _ => "duplicate key",
    }
}

fn parse_envelope_line(raw_line: &str, fields: &mut EnvelopeFields) -> Result<(), SdkError> {
    if raw_line.trim().is_empty() {
        return Ok(());
    }
    let (key, value) = split_envelope_line(raw_line)?;
    apply_envelope_field(fields, key, value)
}

fn split_envelope_line(raw_line: &str) -> Result<(&str, &str), SdkError> {
    let (key, raw_value) = raw_line.split_once('=').ok_or(SdkError::InvalidInput {
        field: "wire_payload",
        reason: "line must contain key=value",
    })?;
    Ok((key, raw_value.trim_end_matches('\r')))
}

fn apply_envelope_field(
    fields: &mut EnvelopeFields,
    key: &str,
    value: &str,
) -> Result<(), SdkError> {
    match key {
        "from" => set_once(&mut fields.from, value, "wire_payload", "from"),
        "to" => set_once(&mut fields.to, value, "wire_payload", "to"),
        "nonce" => set_nonce(&mut fields.nonce, value, "nonce"),
        "state_hash" => set_once(&mut fields.state_hash, value, "wire_payload", "state_hash"),
        "body" => set_once(&mut fields.body, value, "wire_payload", "body"),
        "signer_public_key" => set_once(
            &mut fields.signer_public_key,
            value,
            "wire_payload",
            "signer_public_key",
        ),
        "signature" => set_once(&mut fields.signature, value, "wire_payload", "signature"),
        _ => Err(SdkError::InvalidInput {
            field: "wire_payload",
            reason: "unknown key",
        }),
    }
}
