use super::SdkError;

pub(super) fn parse_unmasked_text_frame_payload(frame: &[u8]) -> Result<&[u8], SdkError> {
    if frame.len() < 2 {
        return Err(SdkError::TransportFailure(
            "service websocket response missing event frame",
        ));
    }
    if frame[0] != 0x81 {
        return Err(SdkError::TransportFailure(
            "service websocket response frame opcode unsupported",
        ));
    }
    if frame[1] & 0x80 != 0 {
        return Err(SdkError::TransportFailure(
            "service websocket response frame unexpectedly masked",
        ));
    }

    let length_marker = frame[1] & 0x7f;
    let (payload_offset, payload_len) = match length_marker {
        0..=125 => (2_usize, length_marker as usize),
        126 => parse_u16_payload_length(frame)?,
        127 => parse_u64_payload_length(frame)?,
        _ => {
            return Err(SdkError::TransportFailure(
                "service websocket response frame payload length unsupported",
            ));
        }
    };
    let frame_end = payload_offset
        .checked_add(payload_len)
        .ok_or(SdkError::TransportFailure(
            "service websocket response frame payload too large",
        ))?;
    if frame.len() < frame_end {
        return Err(SdkError::TransportFailure(
            "service websocket response frame payload truncated",
        ));
    }
    Ok(&frame[payload_offset..frame_end])
}

fn parse_u16_payload_length(frame: &[u8]) -> Result<(usize, usize), SdkError> {
    if frame.len() < 4 {
        return Err(SdkError::TransportFailure(
            "service websocket response frame payload truncated",
        ));
    }
    let payload_len = u16::from_be_bytes([frame[2], frame[3]]) as usize;
    Ok((4, payload_len))
}

fn parse_u64_payload_length(frame: &[u8]) -> Result<(usize, usize), SdkError> {
    if frame.len() < 10 {
        return Err(SdkError::TransportFailure(
            "service websocket response frame payload truncated",
        ));
    }
    let encoded_len = u64::from_be_bytes([
        frame[2], frame[3], frame[4], frame[5], frame[6], frame[7], frame[8], frame[9],
    ]);
    let payload_len = usize::try_from(encoded_len).map_err(|_| {
        SdkError::TransportFailure("service websocket response frame payload too large")
    })?;
    Ok((10, payload_len))
}
