use crate::{AgentDid, SdkError};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const DEFAULT_CONNECT_RETRIES: u32 = 20;
const DEFAULT_RETRY_DELAY_MILLIS: u64 = 100;
const DEFAULT_MAX_WIRE_BYTES: usize = 32 * 1024;

/// Deterministic signed envelope transported over TCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpSignedEnvelope {
    /// Sender DID.
    pub from: AgentDid,
    /// Recipient DID.
    pub to: AgentDid,
    /// Monotonic nonce.
    pub nonce: u64,
    /// Runtime state hash marker.
    pub state_hash: String,
    /// Message body.
    pub body: String,
    /// Deterministic signature marker.
    pub signature: String,
}

impl TcpSignedEnvelope {
    /// Builds and signs a deterministic TCP envelope.
    pub fn new(
        from: AgentDid,
        to: AgentDid,
        nonce: u64,
        state_hash: impl Into<String>,
        body: impl Into<String>,
    ) -> Result<Self, SdkError> {
        let state_hash = state_hash.into();
        let body = body.into();
        let signature = signature_for_fields(from.as_str(), nonce, &state_hash, &body);

        let envelope = Self {
            from,
            to,
            nonce,
            state_hash,
            body,
            signature,
        };
        envelope.verify_integrity()?;
        Ok(envelope)
    }

    /// Returns deterministic wire payload in canonical field order.
    pub fn to_wire_payload(&self) -> String {
        format!(
            "from={}\nto={}\nnonce={}\nstate_hash={}\nbody={}\nsignature={}\n",
            self.from.as_str(),
            self.to.as_str(),
            self.nonce,
            self.state_hash,
            self.body,
            self.signature
        )
    }

    /// Parses and verifies deterministic wire payload.
    pub fn parse_wire_payload(payload: &str) -> Result<Self, SdkError> {
        let mut from: Option<String> = None;
        let mut to: Option<String> = None;
        let mut nonce: Option<u64> = None;
        let mut state_hash: Option<String> = None;
        let mut body: Option<String> = None;
        let mut signature: Option<String> = None;

        for raw_line in payload.lines() {
            if raw_line.trim().is_empty() {
                continue;
            }
            let (key, raw_value) = raw_line.split_once('=').ok_or(SdkError::InvalidInput {
                field: "wire_payload",
                reason: "line must contain key=value",
            })?;
            let value = raw_value.trim_end_matches('\r');

            match key {
                "from" => {
                    if from.is_some() {
                        return Err(SdkError::InvalidInput {
                            field: "wire_payload",
                            reason: "duplicate key: from",
                        });
                    }
                    from = Some(value.to_owned());
                }
                "to" => {
                    if to.is_some() {
                        return Err(SdkError::InvalidInput {
                            field: "wire_payload",
                            reason: "duplicate key: to",
                        });
                    }
                    to = Some(value.to_owned());
                }
                "nonce" => {
                    if nonce.is_some() {
                        return Err(SdkError::InvalidInput {
                            field: "wire_payload",
                            reason: "duplicate key: nonce",
                        });
                    }
                    nonce = Some(value.parse::<u64>().map_err(|_| SdkError::InvalidInput {
                        field: "nonce",
                        reason: "must be an unsigned integer",
                    })?);
                }
                "state_hash" => {
                    if state_hash.is_some() {
                        return Err(SdkError::InvalidInput {
                            field: "wire_payload",
                            reason: "duplicate key: state_hash",
                        });
                    }
                    state_hash = Some(value.to_owned());
                }
                "body" => {
                    if body.is_some() {
                        return Err(SdkError::InvalidInput {
                            field: "wire_payload",
                            reason: "duplicate key: body",
                        });
                    }
                    body = Some(value.to_owned());
                }
                "signature" => {
                    if signature.is_some() {
                        return Err(SdkError::InvalidInput {
                            field: "wire_payload",
                            reason: "duplicate key: signature",
                        });
                    }
                    signature = Some(value.to_owned());
                }
                _ => {
                    return Err(SdkError::InvalidInput {
                        field: "wire_payload",
                        reason: "unknown key",
                    });
                }
            }
        }

        let envelope = Self {
            from: AgentDid::parse(from.ok_or(SdkError::InvalidInput {
                field: "from",
                reason: "missing required key",
            })?)?,
            to: AgentDid::parse(to.ok_or(SdkError::InvalidInput {
                field: "to",
                reason: "missing required key",
            })?)?,
            nonce: nonce.ok_or(SdkError::InvalidInput {
                field: "nonce",
                reason: "missing required key",
            })?,
            state_hash: state_hash.ok_or(SdkError::InvalidInput {
                field: "state_hash",
                reason: "missing required key",
            })?,
            body: body.ok_or(SdkError::InvalidInput {
                field: "body",
                reason: "missing required key",
            })?,
            signature: signature.ok_or(SdkError::InvalidInput {
                field: "signature",
                reason: "missing required key",
            })?,
        };

        envelope.verify_integrity()?;
        Ok(envelope)
    }

    /// Verifies payload shape and deterministic signature.
    pub fn verify_integrity(&self) -> Result<(), SdkError> {
        if self.state_hash.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "state_hash",
                reason: "must not be empty",
            });
        }
        if self.body.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "body",
                reason: "must not be empty",
            });
        }
        if self.state_hash.contains('\n') || self.state_hash.contains('\r') {
            return Err(SdkError::InvalidInput {
                field: "state_hash",
                reason: "must be single-line",
            });
        }
        if self.body.contains('\n') || self.body.contains('\r') {
            return Err(SdkError::InvalidInput {
                field: "body",
                reason: "must be single-line",
            });
        }

        let expected = signature_for_fields(
            self.from.as_str(),
            self.nonce,
            self.state_hash.as_str(),
            self.body.as_str(),
        );
        if self.signature != expected {
            return Err(SdkError::InvalidInput {
                field: "signature",
                reason: "does not match deterministic envelope fields",
            });
        }
        Ok(())
    }
}

/// Deterministic signature marker for TCP envelope fields.
pub fn signature_for_fields(from: &str, nonce: u64, state_hash: &str, body: &str) -> String {
    format!(
        "sig:ed25519:baseline-v1:{from}:{nonce}:{state_hash}:{}",
        body.len()
    )
}

/// TCP transport configuration for local relay flows.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Sets deterministic connect retry count.
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

    /// Sets deterministic retry delay in milliseconds.
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

    /// Sets maximum wire payload size in bytes.
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

    /// Returns configured socket address.
    pub fn addr(&self) -> &str {
        &self.addr
    }
}

/// Result of receiving one envelope over TCP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TcpReceivedEnvelope {
    /// Parsed and verified envelope payload.
    pub envelope: TcpSignedEnvelope,
    /// Remote peer socket address.
    pub peer_addr: String,
}

/// Minimal TCP relay adapter for deterministic local transport harnesses.
#[derive(Debug, Clone)]
pub struct TcpTransportAdapter {
    config: TcpTransportConfig,
}

impl TcpTransportAdapter {
    /// Creates a new adapter for the provided transport config.
    pub fn new(config: TcpTransportConfig) -> Self {
        Self { config }
    }

    /// Sends a deterministic envelope to the configured TCP endpoint.
    pub fn send(&self, envelope: &TcpSignedEnvelope) -> Result<(), SdkError> {
        envelope.verify_integrity()?;
        let payload = envelope.to_wire_payload();

        let mut stream = self.connect_with_retry()?;
        stream
            .write_all(payload.as_bytes())
            .map_err(|_| SdkError::TransportFailure("tcp write failed"))?;
        stream
            .flush()
            .map_err(|_| SdkError::TransportFailure("tcp flush failed"))?;
        stream
            .shutdown(Shutdown::Write)
            .map_err(|_| SdkError::TransportFailure("tcp shutdown failed"))?;
        Ok(())
    }

    /// Binds, accepts one inbound message, and parses/verifies envelope payload.
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

        TcpSignedEnvelope::parse_wire_payload(payload.as_str())
    }
}
