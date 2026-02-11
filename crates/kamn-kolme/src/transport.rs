//! Transport contracts for Kolme runtime-commit submissions.

use std::error::Error;
use std::fmt;

/// Runtime-commit transport request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportRequest {
    /// Target endpoint path or URL.
    pub endpoint: String,
    /// Serialized payload body.
    pub body: Vec<u8>,
}

impl TransportRequest {
    /// Creates a new request envelope.
    pub fn new(endpoint: impl Into<String>, body: Vec<u8>) -> Self {
        Self {
            endpoint: endpoint.into(),
            body,
        }
    }
}

/// Runtime-commit transport response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportResponse {
    /// Transport status code.
    pub status: u16,
    /// Response payload body.
    pub body: Vec<u8>,
}

impl TransportResponse {
    /// Creates a new transport response.
    pub fn new(status: u16, body: Vec<u8>) -> Self {
        Self { status, body }
    }
}

/// Transport-level error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Endpoint is missing.
    EmptyEndpoint,
    /// Upstream returned failure status.
    RejectedStatus(u16),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEndpoint => f.write_str("transport endpoint must not be empty"),
            Self::RejectedStatus(status) => write!(f, "transport rejected with status {status}"),
        }
    }
}

impl Error for TransportError {}

/// Transport boundary for runtime-commit submissions.
pub trait KolmeTransport {
    /// Submits a request and returns a response envelope.
    fn submit(&self, request: TransportRequest) -> Result<TransportResponse, TransportError>;
}

/// Deterministic in-memory transport used by scaffold tests.
#[derive(Debug, Default, Clone, Copy)]
pub struct EchoTransport;

impl KolmeTransport for EchoTransport {
    fn submit(&self, request: TransportRequest) -> Result<TransportResponse, TransportError> {
        if request.endpoint.trim().is_empty() {
            return Err(TransportError::EmptyEndpoint);
        }
        Ok(TransportResponse::new(200, request.body))
    }
}

#[cfg(test)]
mod tests {
    use super::{EchoTransport, KolmeTransport, TransportError, TransportRequest};

    #[test]
    fn unit_echo_transport_requires_endpoint() {
        let transport = EchoTransport;
        let request = TransportRequest::new("", b"payload".to_vec());
        assert_eq!(
            transport.submit(request),
            Err(TransportError::EmptyEndpoint)
        );
    }

    #[test]
    fn unit_echo_transport_roundtrips_payload() {
        let transport = EchoTransport;
        let payload = b"bridge".to_vec();
        let response = transport
            .submit(TransportRequest::new(
                "http://127.0.0.1:8080",
                payload.clone(),
            ))
            .expect("submit should succeed");
        assert_eq!(response.status, 200);
        assert_eq!(response.body, payload);
    }
}
