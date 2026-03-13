use super::super::support::{TCP_HANDSHAKE_PROFILE, TCP_HANDSHAKE_VERSION};
use super::field_support::{require_exact, required_nonce, required_string, set_nonce, set_once};
use super::TcpHandshakeFrame;
use crate::{AgentDid, SdkError};

pub(super) struct HandshakeFields {
    frame: Option<String>,
    version: Option<String>,
    profile: Option<String>,
    from: Option<String>,
    to: Option<String>,
    nonce: Option<u64>,
    signer_public_key: Option<String>,
    signature: Option<String>,
}

struct ParsedHandshakeFields {
    frame: String,
    version: String,
    profile: String,
    from: String,
    to: String,
    nonce: u64,
    signer_public_key: String,
    signature: String,
}

impl HandshakeFields {
    fn new() -> Self {
        Self {
            frame: None,
            version: None,
            profile: None,
            from: None,
            to: None,
            nonce: None,
            signer_public_key: None,
            signature: None,
        }
    }
}

pub(super) fn parse_handshake_fields(payload: &str) -> Result<HandshakeFields, SdkError> {
    let mut fields = HandshakeFields::new();
    for raw_line in payload.lines() {
        parse_handshake_line(raw_line, &mut fields)?;
    }
    Ok(fields)
}

pub(super) fn build_handshake_frame(
    fields: HandshakeFields,
) -> Result<TcpHandshakeFrame, SdkError> {
    let parsed = extract_required_fields(fields)?;
    validate_handshake_metadata(&parsed)?;
    Ok(TcpHandshakeFrame {
        from: AgentDid::parse(parsed.from.as_str())?,
        to: AgentDid::parse(parsed.to.as_str())?,
        nonce: parsed.nonce,
        signer_public_key: parsed.signer_public_key,
        signature: parsed.signature,
    })
}

fn parse_handshake_line(raw_line: &str, fields: &mut HandshakeFields) -> Result<(), SdkError> {
    let Some((key, value)) = parse_handshake_line_parts(raw_line)? else {
        return Ok(());
    };
    set_handshake_field(fields, key, value)
}

fn extract_required_fields(fields: HandshakeFields) -> Result<ParsedHandshakeFields, SdkError> {
    Ok(ParsedHandshakeFields {
        frame: required_string(fields.frame, "handshake.frame")?,
        version: required_string(fields.version, "handshake.version")?,
        profile: required_string(fields.profile, "handshake.profile")?,
        from: required_string(fields.from, "handshake.from")?,
        to: required_string(fields.to, "handshake.to")?,
        nonce: required_nonce(fields.nonce, "handshake.nonce")?,
        signer_public_key: required_string(
            fields.signer_public_key,
            "handshake.signer_public_key",
        )?,
        signature: required_string(fields.signature, "handshake.signature")?,
    })
}

fn validate_handshake_metadata(fields: &ParsedHandshakeFields) -> Result<(), SdkError> {
    require_exact(
        fields.frame.clone(),
        "handshake.frame",
        "handshake",
        "must equal handshake",
    )?;
    require_exact(
        fields.version.clone(),
        "handshake.version",
        TCP_HANDSHAKE_VERSION,
        "unsupported handshake version",
    )?;
    require_exact(
        fields.profile.clone(),
        "handshake.profile",
        TCP_HANDSHAKE_PROFILE,
        "unsupported signature profile",
    )
}

fn parse_handshake_line_parts(raw_line: &str) -> Result<Option<(&str, &str)>, SdkError> {
    if raw_line.trim().is_empty() {
        return Ok(None);
    }
    let (key, raw_value) = raw_line.split_once('=').ok_or(SdkError::InvalidInput {
        field: "handshake_frame",
        reason: "line must contain key=value",
    })?;
    Ok(Some((key, raw_value.trim_end_matches('\r'))))
}

fn set_handshake_field(
    fields: &mut HandshakeFields,
    key: &str,
    value: &str,
) -> Result<(), SdkError> {
    match key {
        "frame" => set_once(&mut fields.frame, value, "handshake_frame", "frame"),
        "version" => set_once(&mut fields.version, value, "handshake_frame", "version"),
        "profile" => set_once(&mut fields.profile, value, "handshake_frame", "profile"),
        "from" => set_once(&mut fields.from, value, "handshake_frame", "from"),
        "to" => set_once(&mut fields.to, value, "handshake_frame", "to"),
        "nonce" => set_nonce(&mut fields.nonce, value, "handshake.nonce"),
        "signer_public_key" => set_once(
            &mut fields.signer_public_key,
            value,
            "handshake_frame",
            "signer_public_key",
        ),
        "signature" => set_once(&mut fields.signature, value, "handshake_frame", "signature"),
        _ => Err(SdkError::InvalidInput {
            field: "handshake_frame",
            reason: "unknown key",
        }),
    }
}
