//! Deterministic HTTP/TLS transport implementation for runtime commit submit/finality calls.

use super::{
    classify_kolme_tls_failure_reason, classify_kolme_transport_io_error,
    compose_kolme_finality_status_path, is_kolme_broadcast_submit_path_contract,
    is_kolme_valid_block_lookup_height_contract, is_kolme_valid_http_response_bytes_input_contract,
    is_kolme_valid_http_transport_timeout_seconds_contract,
    is_kolme_valid_transport_idempotency_key_input_contract,
    is_kolme_valid_transport_wire_payload_input_contract,
    normalize_kolme_broadcast_payload_contract,
    normalize_kolme_broadcast_submit_path_input_contract,
    normalize_kolme_transport_idempotency_key_input_contract,
    parse_kolme_authorization_header_value, parse_kolme_http_endpoint,
    parse_kolme_http_response_body, render_kolme_block_path,
    resolve_kolme_tls_ca_file_env_result_contract, KamnKolmeHttpResponsePolicyError,
    KamnKolmeHttpScheme, KamnKolmeParsedHttpEndpoint, KamnKolmeTlsPolicyError,
    KamnKolmeTransportRequestPolicyError, KolmeApiBroadcastRequest, KolmeApiBroadcastResponse,
    KolmeApiNextNonceRequest, KolmeApiNextNonceResponse, KolmeRuntimeCommitError,
};
use kamn_kolme::{
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitFinalityTransport,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderTransport,
};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::net::IpAddr;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Duration;

type ParsedHttpEndpoint = KamnKolmeParsedHttpEndpoint;

type HttpScheme = KamnKolmeHttpScheme;

const HTTP_CONNECTION_POOL_PER_ENDPOINT_CAPACITY: usize = 4;

/// Deterministic HTTP/TLS transport implementation for runtime commit submit/finality calls.
#[derive(Debug, Clone)]
pub struct KolmeRuntimeCommitHttpTransport {
    timeout_seconds: u64,
    authorization_header: Option<String>,
    http_connection_pool: Arc<Mutex<BTreeMap<String, Vec<TcpStream>>>>,
}

impl PartialEq for KolmeRuntimeCommitHttpTransport {
    fn eq(&self, other: &Self) -> bool {
        self.timeout_seconds == other.timeout_seconds
            && self.authorization_header == other.authorization_header
    }
}

impl Eq for KolmeRuntimeCommitHttpTransport {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HttpResponseMetadata {
    header_end: usize,
    content_length: Option<usize>,
    connection_keep_alive: bool,
}

impl KolmeRuntimeCommitHttpTransport {
    /// Builds a concrete HTTP transport with deterministic timeout validation.
    pub fn new(timeout_seconds: u64) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_http_transport_timeout_seconds_contract(timeout_seconds) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "transport_timeout_seconds",
                reason: "must be positive",
            });
        }
        Ok(Self {
            timeout_seconds,
            authorization_header: None,
            http_connection_pool: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }

    /// Builds a concrete HTTP transport with deterministic authorization header configuration.
    pub fn new_with_authorization(
        timeout_seconds: u64,
        authorization_header: &str,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let mut transport = Self::new(timeout_seconds)?;
        transport.authorization_header = Some(
            parse_kolme_authorization_header_value(authorization_header).map_err(|error| {
                match error {
                    KamnKolmeTransportRequestPolicyError::InvalidRequest { field, reason } => {
                        KolmeRuntimeCommitError::InvalidRequest { field, reason }
                    }
                }
            })?,
        );
        Ok(transport)
    }

    /// Fetches one typed nonce response from `/get-next-nonce`.
    pub fn fetch_next_nonce(
        &mut self,
        base_url: &str,
        nonce_path: &str,
        request: &KolmeApiNextNonceRequest,
    ) -> Result<KolmeApiNextNonceResponse, KolmeRuntimeCommitProviderError> {
        let path = request.query_path(nonce_path);
        let response = self.execute_request(base_url, path.as_str(), "GET", None, &[])?;
        KolmeApiNextNonceResponse::parse_json(response.as_str())
    }

    /// Submits one typed broadcast request to `/broadcast`.
    pub fn submit_broadcast_request(
        &mut self,
        base_url: &str,
        submit_path: &str,
        request: &KolmeApiBroadcastRequest,
        idempotency_key: &str,
    ) -> Result<KolmeApiBroadcastResponse, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_transport_idempotency_key_input_contract(idempotency_key) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "idempotency_key must not be empty".to_owned(),
            });
        }
        let idempotency_key =
            normalize_kolme_transport_idempotency_key_input_contract(idempotency_key);
        let submit_path = normalize_kolme_broadcast_submit_path_input_contract(submit_path);
        let payload = request.to_json_payload();
        let response = self.execute_request(
            base_url,
            submit_path,
            "PUT",
            Some(payload.as_str()),
            &[
                ("Content-Type", "application/json"),
                ("X-Idempotency-Key", idempotency_key),
            ],
        )?;
        KolmeApiBroadcastResponse::parse_json(response.as_str())
    }

    fn execute_request(
        &self,
        base_url: &str,
        path: &str,
        method: &str,
        body: Option<&str>,
        headers: &[(&str, &str)],
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        let endpoint = parse_kolme_http_endpoint(base_url, path).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            }
        })?;
        let payload = body.unwrap_or("");
        let connection_header = match endpoint.scheme {
            HttpScheme::Http => "keep-alive",
            HttpScheme::Https => "close",
        };
        let mut request = format!(
            "{method} {} HTTP/1.1\r\nHost: {}\r\nConnection: {connection_header}\r\n",
            endpoint.target_path, endpoint.host_header,
        );
        for (header_name, header_value) in headers {
            request.push_str(header_name);
            request.push_str(": ");
            request.push_str(header_value);
            request.push_str("\r\n");
        }
        if let Some(authorization_header) = self.authorization_header.as_deref() {
            request.push_str("Authorization: ");
            request.push_str(authorization_header);
            request.push_str("\r\n");
        }
        if body.is_some() {
            request.push_str(format!("Content-Length: {}\r\n", payload.len()).as_str());
        }
        request.push_str("\r\n");
        if body.is_some() {
            request.push_str(payload);
        }

        let response_bytes = match endpoint.scheme {
            HttpScheme::Http => self.execute_http_request(endpoint, request.as_bytes())?,
            HttpScheme::Https => self.execute_https_request(endpoint, request.as_bytes())?,
        };
        parse_kolme_http_response_body(response_bytes).map_err(|error| match error {
            KamnKolmeHttpResponsePolicyError::Timeout => KolmeRuntimeCommitProviderError::Timeout,
            KamnKolmeHttpResponsePolicyError::Unavailable { reason } => {
                KolmeRuntimeCommitProviderError::Unavailable { reason }
            }
            KamnKolmeHttpResponsePolicyError::Malformed { reason } => {
                KolmeRuntimeCommitProviderError::MalformedResponse { reason }
            }
        })
    }

    fn execute_http_request(
        &self,
        endpoint: ParsedHttpEndpoint,
        request: &[u8],
    ) -> Result<Vec<u8>, KolmeRuntimeCommitProviderError> {
        let timeout = Duration::from_secs(self.timeout_seconds);
        let pool_key = format!("{}:{}", endpoint.host, endpoint.port);
        let mut stream =
            if let Some(pooled_stream) = self.take_pooled_http_stream(pool_key.as_str()) {
                pooled_stream
            } else {
                self.connect_http_stream(endpoint.host.as_str(), endpoint.port)?
            };
        Self::configure_http_stream_timeout(&stream, timeout)?;
        stream.write_all(request).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;

        let (response_bytes, keep_alive) = Self::read_http_response_bytes(&mut stream)?;
        if keep_alive {
            self.return_pooled_http_stream(pool_key, stream);
        }
        Ok(response_bytes)
    }

    fn connect_http_stream(
        &self,
        host: &str,
        port: u16,
    ) -> Result<TcpStream, KolmeRuntimeCommitProviderError> {
        TcpStream::connect((host, port)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })
    }

    fn configure_http_stream_timeout(
        stream: &TcpStream,
        timeout: Duration,
    ) -> Result<(), KolmeRuntimeCommitProviderError> {
        stream.set_read_timeout(Some(timeout)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        stream.set_write_timeout(Some(timeout)).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        Ok(())
    }

    fn take_pooled_http_stream(&self, pool_key: &str) -> Option<TcpStream> {
        let mut pool = self
            .http_connection_pool
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut remove_key = false;
        let stream = if let Some(endpoint_pool) = pool.get_mut(pool_key) {
            let stream = endpoint_pool.pop();
            remove_key = endpoint_pool.is_empty();
            stream
        } else {
            None
        };
        if remove_key {
            pool.remove(pool_key);
        }
        stream
    }

    fn return_pooled_http_stream(&self, pool_key: String, stream: TcpStream) {
        let mut pool = self
            .http_connection_pool
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let endpoint_pool = pool.entry(pool_key).or_default();
        if endpoint_pool.len() < HTTP_CONNECTION_POOL_PER_ENDPOINT_CAPACITY {
            endpoint_pool.push(stream);
        }
    }

    fn read_http_response_bytes(
        stream: &mut TcpStream,
    ) -> Result<(Vec<u8>, bool), KolmeRuntimeCommitProviderError> {
        let mut response_bytes = Vec::new();
        let mut buffer = [0_u8; 4096];
        let mut metadata: Option<HttpResponseMetadata> = None;

        loop {
            let read_count = stream.read(&mut buffer).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
            if read_count == 0 {
                break;
            }
            response_bytes.extend_from_slice(&buffer[..read_count]);

            if metadata.is_none()
                && Self::find_http_header_boundary(response_bytes.as_slice()).is_some()
            {
                metadata = Some(Self::parse_http_response_metadata(
                    response_bytes.as_slice(),
                )?);
            }

            if let Some(metadata) = metadata {
                if let Some(content_length) = metadata.content_length {
                    let total_length = metadata.header_end.saturating_add(content_length);
                    if response_bytes.len() >= total_length {
                        response_bytes.truncate(total_length);
                        return Ok((response_bytes, metadata.connection_keep_alive));
                    }
                }
            }
        }

        if response_bytes.is_empty() {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "http response body is empty".to_owned(),
            });
        }
        Ok((response_bytes, false))
    }

    fn parse_http_response_metadata(
        response_bytes: &[u8],
    ) -> Result<HttpResponseMetadata, KolmeRuntimeCommitProviderError> {
        let header_end = Self::find_http_header_boundary(response_bytes).ok_or_else(|| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "http response missing header/body separator".to_owned(),
            }
        })?;
        let header_text = std::str::from_utf8(&response_bytes[..header_end]).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!("http response headers are not valid utf-8: {error}"),
            }
        })?;
        let mut content_length = None;
        let mut keep_alive = false;
        for line in header_text.lines().skip(1) {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            if name.eq_ignore_ascii_case("Content-Length") && content_length.is_none() {
                content_length = Some(value.trim().parse::<usize>().map_err(|_| {
                    KolmeRuntimeCommitProviderError::MalformedResponse {
                        reason: "http response content-length is invalid".to_owned(),
                    }
                })?);
            }
            if name.eq_ignore_ascii_case("Connection") {
                for token in value
                    .split(',')
                    .map(|token| token.trim().to_ascii_lowercase())
                {
                    if token == "keep-alive" {
                        keep_alive = true;
                    }
                    if token == "close" {
                        keep_alive = false;
                    }
                }
            }
        }
        Ok(HttpResponseMetadata {
            header_end,
            content_length,
            connection_keep_alive: keep_alive,
        })
    }

    fn find_http_header_boundary(response_bytes: &[u8]) -> Option<usize> {
        response_bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|index| index + 4)
    }

    fn execute_https_request(
        &self,
        endpoint: ParsedHttpEndpoint,
        request: &[u8],
    ) -> Result<Vec<u8>, KolmeRuntimeCommitProviderError> {
        let stream =
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

        let tls_config = self.resolve_tls_client_config()?;
        let server_name = Self::resolve_tls_server_name(endpoint.host.as_str())?;
        let connection = ClientConnection::new(tls_config, server_name).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: Self::classify_rustls_handshake_error(&error),
            }
        })?;
        let mut tls_stream = StreamOwned::new(connection, stream);
        tls_stream
            .write_all(request)
            .map_err(|error| Self::map_tls_io_error(&error))?;
        tls_stream
            .flush()
            .map_err(|error| Self::map_tls_io_error(&error))?;

        let mut response_bytes = Vec::new();
        let mut read_buffer = [0_u8; 4096];
        loop {
            match tls_stream.read(&mut read_buffer) {
                Ok(0) => break,
                Ok(read_count) => response_bytes.extend_from_slice(&read_buffer[..read_count]),
                Err(error) => {
                    if matches!(error.kind(), std::io::ErrorKind::UnexpectedEof)
                        && !response_bytes.is_empty()
                    {
                        break;
                    }
                    return Err(Self::map_tls_io_error(&error));
                }
            }
        }
        if !is_kolme_valid_http_response_bytes_input_contract(response_bytes.as_slice()) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "tls response body is empty".to_owned(),
            });
        }
        Ok(response_bytes)
    }

    fn resolve_tls_client_config(
        &self,
    ) -> Result<Arc<ClientConfig>, KolmeRuntimeCommitProviderError> {
        let configured_ca_file =
            resolve_kolme_tls_ca_file_env_result_contract(std::env::var("KAMN_KOLME_TLS_CA_FILE"))
                .map_err(|error| match error {
                    KamnKolmeTlsPolicyError::Unavailable { reason } => {
                        KolmeRuntimeCommitProviderError::Unavailable { reason }
                    }
                })?;
        let mut root_store = RootCertStore::empty();
        match configured_ca_file {
            Some(ca_file) => {
                let cert_bytes = fs::read(ca_file.as_str()).map_err(|error| {
                    KolmeRuntimeCommitProviderError::Unavailable {
                        reason: format!("tls ca file read failed: {error}"),
                    }
                })?;
                let mut cert_reader = Cursor::new(cert_bytes.as_slice());
                let certs = rustls_pemfile::certs(&mut cert_reader)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|error| KolmeRuntimeCommitProviderError::Unavailable {
                        reason: format!("tls ca file parse failed: {error}"),
                    })?;
                let (added, _) = root_store.add_parsable_certificates(certs);
                if added == 0 {
                    return Err(KolmeRuntimeCommitProviderError::Unavailable {
                        reason: "tls ca file does not contain valid certificates".to_owned(),
                    });
                }
            }
            None => {
                root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            }
        }
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        Ok(Arc::new(config))
    }

    fn resolve_tls_server_name(
        host: &str,
    ) -> Result<ServerName<'static>, KolmeRuntimeCommitProviderError> {
        if let Ok(ip_addr) = host.parse::<IpAddr>() {
            return Ok(ServerName::IpAddress(ip_addr.into()));
        }
        ServerName::try_from(host.to_owned()).map_err(|_| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: "tls handshake failed".to_owned(),
            }
        })
    }

    fn map_tls_io_error(error: &std::io::Error) -> KolmeRuntimeCommitProviderError {
        if let Some(rustls_error) = error
            .get_ref()
            .and_then(|source| source.downcast_ref::<rustls::Error>())
        {
            return KolmeRuntimeCommitProviderError::Unavailable {
                reason: Self::classify_rustls_handshake_error(rustls_error),
            };
        }
        match classify_kolme_transport_io_error(error) {
            kamn_kolme::KolmeTransportIoClassification::Timeout => {
                KolmeRuntimeCommitProviderError::Timeout
            }
            kamn_kolme::KolmeTransportIoClassification::Unavailable { reason } => {
                let classified = classify_kolme_tls_failure_reason(error.to_string().as_str());
                if classified == "tls certificate verification failed"
                    || classified == "tls handshake failed"
                {
                    return KolmeRuntimeCommitProviderError::Unavailable { reason: classified };
                }
                KolmeRuntimeCommitProviderError::Unavailable { reason }
            }
        }
    }

    fn classify_rustls_handshake_error(error: &rustls::Error) -> String {
        match error {
            rustls::Error::InvalidCertificate(_) => {
                "tls certificate verification failed".to_owned()
            }
            _ => "tls handshake failed".to_owned(),
        }
    }
}

impl KolmeRuntimeCommitProviderTransport for KolmeRuntimeCommitHttpTransport {
    fn submit_runtime_commit(
        &mut self,
        base_url: &str,
        submit_path: &str,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_transport_wire_payload_input_contract(wire_payload) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "wire_payload must not be empty".to_owned(),
            });
        }
        if !is_kolme_valid_transport_idempotency_key_input_contract(idempotency_key) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "idempotency_key must not be empty".to_owned(),
            });
        }
        if is_kolme_broadcast_submit_path_contract(submit_path) {
            let payload = normalize_kolme_broadcast_payload_contract(wire_payload, idempotency_key)
                .map_err(|error| KolmeRuntimeCommitProviderError::MalformedResponse {
                    reason: error.to_string(),
                })?;
            return self.execute_request(
                base_url,
                submit_path,
                "PUT",
                Some(payload.as_str()),
                &[
                    ("Content-Type", "application/json"),
                    ("X-Idempotency-Key", idempotency_key),
                ],
            );
        }
        self.execute_request(
            base_url,
            submit_path,
            "POST",
            Some(wire_payload),
            &[
                ("Content-Type", "text/plain"),
                ("X-Idempotency-Key", idempotency_key),
            ],
        )
    }
}

impl KolmeRuntimeCommitFinalityTransport for KolmeRuntimeCommitHttpTransport {
    fn fetch_runtime_commit_finality(
        &mut self,
        base_url: &str,
        status_path: &str,
        commit_id: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        let path = compose_kolme_finality_status_path(status_path, commit_id).map_err(|error| {
            KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: error.to_string(),
            }
        })?;
        self.execute_request(base_url, path.as_str(), "GET", None, &[])
    }
}

impl KolmeRuntimeCommitBlockFallbackTransport for KolmeRuntimeCommitHttpTransport {
    fn fetch_block_by_height(
        &mut self,
        base_url: &str,
        block_path_template: &str,
        height: u64,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        if !is_kolme_valid_block_lookup_height_contract(height) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "block height must be positive".to_owned(),
            });
        }
        let block_path = render_kolme_block_path(block_path_template, height).map_err(|error| {
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: error.to_string(),
            }
        })?;
        self.execute_request(base_url, block_path.as_str(), "GET", None, &[])
    }
}
