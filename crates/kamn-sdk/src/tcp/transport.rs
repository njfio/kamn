use super::envelope::TcpSignedEnvelope;
use super::handshake::TcpHandshakeFrame;
use super::support::{
    is_benign_tcp_shutdown_error, DEFAULT_CONNECT_RETRIES, DEFAULT_MAX_WIRE_BYTES,
    DEFAULT_RETRY_DELAY_MILLIS,
};
use crate::SdkError;
use std::collections::HashMap;
use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "transport/config.rs"]
mod config;
#[path = "transport/read.rs"]
mod read;
#[path = "transport/support.rs"]
mod support;
use config::{
    parse_socket_addr, validate_positive_u32, validate_positive_u64, validate_positive_usize,
};
use read::read_envelope;
use support::serialize_transport_payload;

#[derive(Debug, Default)]
struct TcpReplayGuardState {
    highest_nonce_by_route: HashMap<String, u64>,
}

impl TcpReplayGuardState {
    fn verify_and_record(&mut self, envelope: &TcpSignedEnvelope) -> Result<(), SdkError> {
        let route_key = format!("{}=>{}", envelope.from.as_str(), envelope.to.as_str());
        if let Some(highest_nonce) = self.highest_nonce_by_route.get(&route_key) {
            if envelope.nonce <= *highest_nonce {
                return Err(SdkError::Conflict("tcp handshake replay detected"));
            }
        }
        self.highest_nonce_by_route
            .insert(route_key, envelope.nonce);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// TCP transport configuration for local relay flows.
pub struct TcpTransportConfig {
    addr: String,
    connect_retries: u32,
    retry_delay_millis: u64,
    max_wire_bytes: usize,
}

impl TcpTransportConfig {
    /// Creates a validated TCP transport config.
    pub fn new(addr: &str) -> Result<Self, SdkError> {
        let normalized = addr.trim();
        parse_socket_addr(normalized)?;
        Ok(Self {
            addr: normalized.to_owned(),
            connect_retries: DEFAULT_CONNECT_RETRIES,
            retry_delay_millis: DEFAULT_RETRY_DELAY_MILLIS,
            max_wire_bytes: DEFAULT_MAX_WIRE_BYTES,
        })
    }

    /// Sets deterministic connect retry count.
    pub fn with_connect_retries(mut self, retries: u32) -> Result<Self, SdkError> {
        validate_positive_u32(retries, "transport.connect_retries")?;
        self.connect_retries = retries;
        Ok(self)
    }

    /// Sets deterministic retry delay in milliseconds.
    pub fn with_retry_delay_millis(mut self, delay_millis: u64) -> Result<Self, SdkError> {
        validate_positive_u64(delay_millis, "transport.retry_delay_millis")?;
        self.retry_delay_millis = delay_millis;
        Ok(self)
    }

    /// Sets maximum wire payload size in bytes.
    pub fn with_max_wire_bytes(mut self, max_wire_bytes: usize) -> Result<Self, SdkError> {
        validate_positive_usize(max_wire_bytes, "transport.max_wire_bytes")?;
        self.max_wire_bytes = max_wire_bytes;
        Ok(self)
    }

    /// Returns configured socket address.
    pub fn addr(&self) -> &str {
        &self.addr
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Result of receiving one envelope over TCP.
pub struct TcpReceivedEnvelope {
    /// Parsed and verified envelope payload.
    pub envelope: TcpSignedEnvelope,
    /// Remote peer socket address.
    pub peer_addr: String,
}

#[derive(Debug, Clone)]
/// Minimal TCP relay adapter for deterministic local transport harnesses.
pub struct TcpTransportAdapter {
    config: TcpTransportConfig,
    replay_guard_state: Arc<Mutex<TcpReplayGuardState>>,
}

impl TcpTransportAdapter {
    /// Creates a new adapter for the provided transport config.
    pub fn new(config: TcpTransportConfig) -> Self {
        Self {
            config,
            replay_guard_state: Arc::new(Mutex::new(TcpReplayGuardState::default())),
        }
    }

    /// Sends a deterministic envelope to the configured TCP endpoint.
    pub fn send(&self, envelope: &TcpSignedEnvelope) -> Result<(), SdkError> {
        envelope.verify_integrity()?;
        let handshake = TcpHandshakeFrame::from_envelope(envelope);
        let payload = serialize_transport_payload(&handshake, envelope);
        let mut stream = self.connect_with_retry()?;
        stream
            .write_all(payload.as_bytes())
            .map_err(|_| SdkError::TransportFailure("tcp write failed"))?;
        stream
            .flush()
            .map_err(|_| SdkError::TransportFailure("tcp flush failed"))?;
        if let Err(error) = stream.shutdown(Shutdown::Write) {
            if !is_benign_tcp_shutdown_error(&error) {
                return Err(SdkError::TransportFailure("tcp shutdown failed"));
            }
        }
        Ok(())
    }

    /// Binds, accepts one inbound message, and parses/verifies envelope payload.
    pub fn listen_once(&self) -> Result<TcpReceivedEnvelope, SdkError> {
        let listener = TcpListener::bind(self.config.addr.as_str())
            .map_err(|_| SdkError::TransportFailure("tcp bind failed"))?;
        let (stream, peer_addr) = listener
            .accept()
            .map_err(|_| SdkError::TransportFailure("tcp accept failed"))?;
        let envelope = read_envelope(
            stream,
            self.config.max_wire_bytes,
            &self.replay_guard_state,
            Self::verify_and_record_guard,
        )?;
        Ok(TcpReceivedEnvelope {
            envelope,
            peer_addr: peer_addr.to_string(),
        })
    }

    fn connect_with_retry(&self) -> Result<TcpStream, SdkError> {
        for attempt in 0..self.config.connect_retries {
            match TcpStream::connect(self.config.addr.as_str()) {
                Ok(stream) => return Ok(stream),
                Err(_) => {
                    if attempt + 1 == self.config.connect_retries {
                        break;
                    }
                    thread::sleep(Duration::from_millis(self.config.retry_delay_millis));
                }
            }
        }
        Err(SdkError::TransportFailure("tcp connect failed"))
    }

    fn verify_and_record_guard(
        replay_guard_state: &Arc<Mutex<TcpReplayGuardState>>,
        envelope: &TcpSignedEnvelope,
    ) -> Result<(), SdkError> {
        let mut guard = replay_guard_state
            .lock()
            .map_err(|_| SdkError::TransportFailure("tcp replay guard lock poisoned"))?;
        guard.verify_and_record(envelope)
    }
}
