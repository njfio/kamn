use super::envelope::TcpSignedEnvelope;
use super::handshake::TcpHandshakeFrame;
use super::support::{
    is_benign_tcp_shutdown_error, split_transport_payload, DEFAULT_CONNECT_RETRIES,
    DEFAULT_MAX_WIRE_BYTES, DEFAULT_RETRY_DELAY_MILLIS,
};
use crate::SdkError;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "transport/support.rs"]
mod support;
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
pub struct TcpTransportConfig {
    addr: String,
    connect_retries: u32,
    retry_delay_millis: u64,
    max_wire_bytes: usize,
}

impl TcpTransportConfig {
    pub fn new(addr: &str) -> Result<Self, SdkError> {
        let normalized = addr.trim();
        let _parsed: SocketAddr = normalized.parse().map_err(|_| SdkError::InvalidInput {
            field: "transport.addr",
            reason: "must be a valid host:port socket address",
        })?;
        Ok(Self {
            addr: normalized.to_owned(),
            connect_retries: DEFAULT_CONNECT_RETRIES,
            retry_delay_millis: DEFAULT_RETRY_DELAY_MILLIS,
            max_wire_bytes: DEFAULT_MAX_WIRE_BYTES,
        })
    }

    pub fn with_connect_retries(mut self, retries: u32) -> Result<Self, SdkError> {
        if retries == 0 {
            return Err(SdkError::InvalidInput {
                field: "transport.connect_retries",
                reason: "must be greater than zero",
            });
        }
        self.connect_retries = retries;
        Ok(self)
    }

    pub fn with_retry_delay_millis(mut self, delay_millis: u64) -> Result<Self, SdkError> {
        if delay_millis == 0 {
            return Err(SdkError::InvalidInput {
                field: "transport.retry_delay_millis",
                reason: "must be greater than zero",
            });
        }
        self.retry_delay_millis = delay_millis;
        Ok(self)
    }

    pub fn with_max_wire_bytes(mut self, max_wire_bytes: usize) -> Result<Self, SdkError> {
        if max_wire_bytes == 0 {
            return Err(SdkError::InvalidInput {
                field: "transport.max_wire_bytes",
                reason: "must be greater than zero",
            });
        }
        self.max_wire_bytes = max_wire_bytes;
        Ok(self)
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpReceivedEnvelope {
    pub envelope: TcpSignedEnvelope,
    pub peer_addr: String,
}

#[derive(Debug, Clone)]
pub struct TcpTransportAdapter {
    config: TcpTransportConfig,
    replay_guard_state: Arc<Mutex<TcpReplayGuardState>>,
}

impl TcpTransportAdapter {
    pub fn new(config: TcpTransportConfig) -> Self {
        Self {
            config,
            replay_guard_state: Arc::new(Mutex::new(TcpReplayGuardState::default())),
        }
    }

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

    pub fn listen_once(&self) -> Result<TcpReceivedEnvelope, SdkError> {
        let listener = TcpListener::bind(self.config.addr.as_str())
            .map_err(|_| SdkError::TransportFailure("tcp bind failed"))?;
        let (stream, peer_addr) = listener
            .accept()
            .map_err(|_| SdkError::TransportFailure("tcp accept failed"))?;
        let envelope = self.read_envelope(stream)?;
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

    fn read_envelope(&self, stream: TcpStream) -> Result<TcpSignedEnvelope, SdkError> {
        let mut payload = String::new();
        let mut limited_reader = stream.take((self.config.max_wire_bytes + 1) as u64);
        limited_reader
            .read_to_string(&mut payload)
            .map_err(|_| SdkError::TransportFailure("tcp read failed"))?;
        if payload.len() > self.config.max_wire_bytes {
            return Err(SdkError::InvalidInput {
                field: "wire_payload",
                reason: "exceeds max wire bytes",
            });
        }
        let (handshake_payload, envelope_payload) = split_transport_payload(payload.as_str())?;
        let handshake = TcpHandshakeFrame::parse_wire_payload(handshake_payload)?;
        let envelope = TcpSignedEnvelope::parse_wire_payload(envelope_payload)?;
        handshake.verify_matches_envelope(&envelope)?;
        self.verify_and_record_replay_guard(&envelope)?;
        Ok(envelope)
    }

    fn verify_and_record_replay_guard(&self, envelope: &TcpSignedEnvelope) -> Result<(), SdkError> {
        let mut guard = self
            .replay_guard_state
            .lock()
            .map_err(|_| SdkError::TransportFailure("tcp replay guard lock poisoned"))?;
        guard.verify_and_record(envelope)
    }
}
