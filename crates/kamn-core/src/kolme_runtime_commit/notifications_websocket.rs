//! Websocket notifications transport connector/connection implementation.

use super::{
    classify_kolme_transport_io_error, find_kolme_http_header_boundary,
    is_kolme_valid_websocket_timeout_seconds_contract, parse_kolme_websocket_endpoint,
    try_take_kolme_websocket_frame, validate_kolme_websocket_handshake_response,
    KamnKolmeWebsocketFrame, KamnKolmeWebsocketPolicyError, KolmeRuntimeCommitError,
};
use kamn_kolme::{
    KolmeRuntimeCommitNotificationsConnection, KolmeRuntimeCommitNotificationsConnector,
    KolmeRuntimeCommitProviderError,
};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Minimal websocket connector for Kolme `/notifications` consumption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitWebsocketConnector {
    timeout_seconds: u64,
}

impl KolmeRuntimeCommitWebsocketConnector {
    /// Builds a websocket connector with deterministic timeout validation.
    pub fn new(timeout_seconds: u64) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_websocket_timeout_seconds_contract(timeout_seconds) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "notifications_timeout_seconds",
                reason: "must be positive",
            });
        }
        Ok(Self { timeout_seconds })
    }
}

/// Websocket connection implementation used by the default notifications connector.
#[derive(Debug)]
pub struct KolmeRuntimeCommitWebsocketConnection {
    stream: TcpStream,
    read_buffer: Vec<u8>,
}

impl KolmeRuntimeCommitWebsocketConnection {
    fn new(stream: TcpStream, read_buffer: Vec<u8>) -> Self {
        Self {
            stream,
            read_buffer,
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnection for KolmeRuntimeCommitWebsocketConnection {
    fn read_text_message(&mut self) -> Result<Option<String>, KolmeRuntimeCommitProviderError> {
        loop {
            if let Some(frame) = try_take_kolme_websocket_frame(&mut self.read_buffer).map_err(
                |error| match error {
                    KamnKolmeWebsocketPolicyError::Unavailable { reason } => {
                        KolmeRuntimeCommitProviderError::Unavailable { reason }
                    }
                    KamnKolmeWebsocketPolicyError::Malformed { reason } => {
                        KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                    }
                },
            )? {
                match frame {
                    KamnKolmeWebsocketFrame::Text(payload_bytes) => {
                        let payload = String::from_utf8(payload_bytes).map_err(|error| {
                            KolmeRuntimeCommitProviderError::MalformedResponse {
                                reason: format!(
                                    "websocket text payload is not valid utf-8: {error}"
                                ),
                            }
                        })?;
                        return Ok(Some(payload));
                    }
                    KamnKolmeWebsocketFrame::Close => return Ok(None),
                    KamnKolmeWebsocketFrame::Ping | KamnKolmeWebsocketFrame::Pong => continue,
                }
            }

            let mut chunk = [0_u8; 1024];
            let read = self.stream.read(&mut chunk).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
            if read == 0 {
                return Ok(None);
            }
            self.read_buffer.extend_from_slice(&chunk[..read]);
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnector for KolmeRuntimeCommitWebsocketConnector {
    type Connection = KolmeRuntimeCommitWebsocketConnection;

    fn connect(
        &mut self,
        notifications_url: &str,
    ) -> Result<Self::Connection, KolmeRuntimeCommitProviderError> {
        let endpoint = parse_kolme_websocket_endpoint(notifications_url).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            }
        })?;
        if endpoint.secure {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "wss:// notifications are not supported by this transport".to_owned(),
            });
        }

        let mut stream =
            TcpStream::connect((endpoint.host.as_str(), endpoint.port)).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
        let timeout = Duration::from_secs(self.timeout_seconds);
        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;

        let handshake = format!(
            concat!(
                "GET {} HTTP/1.1\r\n",
                "Host: {}\r\n",
                "Upgrade: websocket\r\n",
                "Connection: Upgrade\r\n",
                "Sec-WebSocket-Key: {}\r\n",
                "Sec-WebSocket-Version: 13\r\n",
                "\r\n"
            ),
            endpoint.target_path, endpoint.host_header, "dGhlIHNhbXBsZSBub25jZQ=="
        );
        stream.write_all(handshake.as_bytes()).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;

        let mut response_bytes = Vec::new();
        let header_end = loop {
            if let Some(position) =
                find_kolme_http_header_boundary(&response_bytes).map_err(|error| match error {
                    KamnKolmeWebsocketPolicyError::Unavailable { reason } => {
                        KolmeRuntimeCommitProviderError::Unavailable { reason }
                    }
                    KamnKolmeWebsocketPolicyError::Malformed { reason } => {
                        KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                    }
                })?
            {
                break position;
            }
            let mut chunk = [0_u8; 1024];
            let read = stream.read(&mut chunk).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
            if read == 0 {
                return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: "websocket handshake response is incomplete".to_owned(),
                });
            }
            response_bytes.extend_from_slice(&chunk[..read]);
        };
        let (header_bytes, trailing) = response_bytes.split_at(header_end + 4);
        validate_kolme_websocket_handshake_response(header_bytes).map_err(|error| match error {
            KamnKolmeWebsocketPolicyError::Unavailable { reason } => {
                KolmeRuntimeCommitProviderError::Unavailable { reason }
            }
            KamnKolmeWebsocketPolicyError::Malformed { reason } => {
                KolmeRuntimeCommitProviderError::MalformedResponse { reason }
            }
        })?;
        Ok(KolmeRuntimeCommitWebsocketConnection::new(
            stream,
            trailing.to_vec(),
        ))
    }
}
