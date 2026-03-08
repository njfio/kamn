use crate::{ChannelId, SdkError};

pub(crate) fn channel_name(name: &str) -> Result<&str, SdkError> {
    if name.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "channel_name",
            reason: "must not be empty",
        });
    }
    Ok(name)
}

pub(crate) fn payload(name: &str) -> Result<String, SdkError> {
    let name = channel_name(name)?;
    Ok(serde_json::json!({ "name": name }).to_string())
}

pub(crate) fn channel_id(raw: String) -> Result<ChannelId, SdkError> {
    if raw.trim().is_empty() {
        return Err(SdkError::TransportFailure(
            "service returned empty channel_id in create_channel response",
        ));
    }
    Ok(ChannelId(raw))
}
