//! Websocket protocol policy contracts for Kolme notifications transport.

use std::error::Error;
use std::fmt;

/// Websocket policy error used by handshake/frame parsing contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeWebsocketPolicyError {
    /// Upstream returned an unavailable handshake state.
    Unavailable {
        /// Deterministic policy failure reason.
        reason: String,
    },
    /// Payload/handshake bytes are malformed.
    Malformed {
        /// Deterministic malformed reason.
        reason: String,
    },
}

impl fmt::Display for KolmeWebsocketPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { reason } => f.write_str(reason),
            Self::Malformed { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeWebsocketPolicyError {}

/// Parsed websocket frame variants supported by policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeWebsocketFrame {
    /// UTF-8 text frame payload bytes.
    Text(Vec<u8>),
    /// Ping control frame.
    Ping,
    /// Pong control frame.
    Pong,
    /// Close control frame.
    Close,
}

/// Returns the HTTP header boundary index if present.
///
/// Returns `Ok(None)` when more bytes are required.
pub fn find_http_header_boundary(
    response_bytes: &[u8],
) -> Result<Option<usize>, KolmeWebsocketPolicyError> {
    if let Some(position) = response_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
    {
        return Ok(Some(position));
    }
    if response_bytes.len() > 32 * 1024 {
        return Err(KolmeWebsocketPolicyError::Malformed {
            reason: "websocket handshake response headers are too large".to_owned(),
        });
    }
    Ok(None)
}

/// Validates one websocket handshake response header block.
pub fn validate_websocket_handshake_response(
    header_bytes: &[u8],
) -> Result<(), KolmeWebsocketPolicyError> {
    let header_text = String::from_utf8(header_bytes.to_vec()).map_err(|error| {
        KolmeWebsocketPolicyError::Malformed {
            reason: format!("websocket handshake response is not valid utf-8: {error}"),
        }
    })?;
    let raw_headers = header_text
        .split_once("\r\n\r\n")
        .map(|(headers, _)| headers)
        .unwrap_or(header_text.as_str());

    let mut lines = raw_headers.lines();
    let status_line = lines
        .next()
        .ok_or_else(|| KolmeWebsocketPolicyError::Malformed {
            reason: "websocket handshake response missing status line".to_owned(),
        })?;
    let mut status_parts = status_line.split_whitespace();
    let _http_version =
        status_parts
            .next()
            .ok_or_else(|| KolmeWebsocketPolicyError::Malformed {
                reason: "websocket handshake status line is malformed".to_owned(),
            })?;
    let status_code_raw =
        status_parts
            .next()
            .ok_or_else(|| KolmeWebsocketPolicyError::Malformed {
                reason: "websocket handshake status code is missing".to_owned(),
            })?;
    let status_code =
        status_code_raw
            .parse::<u16>()
            .map_err(|_| KolmeWebsocketPolicyError::Malformed {
                reason: format!("websocket handshake status code is invalid: {status_code_raw}"),
            })?;
    if status_code != 101 {
        return Err(KolmeWebsocketPolicyError::Unavailable {
            reason: format!("websocket handshake rejected with status {status_code}"),
        });
    }

    let mut has_upgrade_websocket = false;
    let mut has_connection_upgrade = false;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("Upgrade")
            && value.trim().to_ascii_lowercase().contains("websocket")
        {
            has_upgrade_websocket = true;
        }
        if name.eq_ignore_ascii_case("Connection")
            && value.trim().to_ascii_lowercase().contains("upgrade")
        {
            has_connection_upgrade = true;
        }
    }
    if !has_upgrade_websocket || !has_connection_upgrade {
        return Err(KolmeWebsocketPolicyError::Malformed {
            reason: "websocket handshake response missing upgrade headers".to_owned(),
        });
    }
    Ok(())
}

/// Extracts one websocket frame from the read buffer.
///
/// Returns `Ok(None)` when more bytes are required to parse one full frame.
pub fn try_take_websocket_frame(
    read_buffer: &mut Vec<u8>,
) -> Result<Option<KolmeWebsocketFrame>, KolmeWebsocketPolicyError> {
    if read_buffer.len() < 2 {
        return Ok(None);
    }
    let first = read_buffer[0];
    let second = read_buffer[1];
    let fin = (first & 0x80) != 0;
    if !fin {
        return Err(KolmeWebsocketPolicyError::Malformed {
            reason: "fragmented websocket frames are not supported".to_owned(),
        });
    }
    let opcode = first & 0x0f;
    let masked = (second & 0x80) != 0;
    let mut payload_len = usize::from(second & 0x7f);
    let mut cursor = 2_usize;

    if payload_len == 126 {
        if read_buffer.len() < cursor + 2 {
            return Ok(None);
        }
        payload_len = usize::from(u16::from_be_bytes([
            read_buffer[cursor],
            read_buffer[cursor + 1],
        ]));
        cursor += 2;
    } else if payload_len == 127 {
        if read_buffer.len() < cursor + 8 {
            return Ok(None);
        }
        let raw = u64::from_be_bytes([
            read_buffer[cursor],
            read_buffer[cursor + 1],
            read_buffer[cursor + 2],
            read_buffer[cursor + 3],
            read_buffer[cursor + 4],
            read_buffer[cursor + 5],
            read_buffer[cursor + 6],
            read_buffer[cursor + 7],
        ]);
        payload_len = usize::try_from(raw).map_err(|_| KolmeWebsocketPolicyError::Malformed {
            reason: "websocket payload length exceeds platform limits".to_owned(),
        })?;
        cursor += 8;
    }

    let masking_key = if masked {
        if read_buffer.len() < cursor + 4 {
            return Ok(None);
        }
        let key = [
            read_buffer[cursor],
            read_buffer[cursor + 1],
            read_buffer[cursor + 2],
            read_buffer[cursor + 3],
        ];
        cursor += 4;
        Some(key)
    } else {
        None
    };

    if read_buffer.len() < cursor + payload_len {
        return Ok(None);
    }
    let mut payload = read_buffer[cursor..cursor + payload_len].to_vec();
    if let Some(masking_key) = masking_key {
        for (index, byte) in payload.iter_mut().enumerate() {
            *byte ^= masking_key[index % 4];
        }
    }
    read_buffer.drain(0..cursor + payload_len);

    let frame = match opcode {
        0x1 => KolmeWebsocketFrame::Text(payload),
        0x8 => KolmeWebsocketFrame::Close,
        0x9 => KolmeWebsocketFrame::Ping,
        0xA => KolmeWebsocketFrame::Pong,
        _ => {
            return Err(KolmeWebsocketPolicyError::Malformed {
                reason: format!("unsupported websocket opcode: {opcode}"),
            })
        }
    };
    Ok(Some(frame))
}

/// Validates websocket connector timeout input in seconds.
pub fn is_valid_websocket_timeout_seconds(timeout_seconds: u64) -> bool {
    timeout_seconds > 0
}

#[cfg(test)]
mod tests {
    use super::{
        find_http_header_boundary, is_valid_websocket_timeout_seconds, try_take_websocket_frame,
        validate_websocket_handshake_response, KolmeWebsocketFrame, KolmeWebsocketPolicyError,
    };

    #[test]
    fn unit_find_http_header_boundary_requires_more_bytes() {
        assert_eq!(
            find_http_header_boundary(b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket"),
            Ok(None)
        );
    }

    #[test]
    fn unit_validate_websocket_handshake_rejects_non_101_status() {
        let header = b"HTTP/1.1 503 Service Unavailable\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        assert_eq!(
            validate_websocket_handshake_response(header),
            Err(KolmeWebsocketPolicyError::Unavailable {
                reason: "websocket handshake rejected with status 503".to_owned(),
            })
        );
    }

    #[test]
    fn unit_try_take_websocket_frame_rejects_fragmented_frame() {
        let mut payload = vec![0x01, 0x01, b'x'];
        assert_eq!(
            try_take_websocket_frame(&mut payload),
            Err(KolmeWebsocketPolicyError::Malformed {
                reason: "fragmented websocket frames are not supported".to_owned(),
            })
        );
    }

    #[test]
    fn functional_try_take_websocket_frame_parses_close_control_frame() {
        let mut payload = vec![0x88, 0x00];
        assert_eq!(
            try_take_websocket_frame(&mut payload)
                .expect("parsing should succeed")
                .expect("frame should be available"),
            KolmeWebsocketFrame::Close
        );
    }

    #[test]
    fn unit_validates_websocket_timeout_seconds_input() {
        assert!(is_valid_websocket_timeout_seconds(2));
        assert!(!is_valid_websocket_timeout_seconds(0));
    }
}
