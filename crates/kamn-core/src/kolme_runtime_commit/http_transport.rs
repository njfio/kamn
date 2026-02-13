//! Dependency-free HTTP transport implementation for runtime commit submit/finality calls.

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
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

type ParsedHttpEndpoint = KamnKolmeParsedHttpEndpoint;

type HttpScheme = KamnKolmeHttpScheme;

/// Dependency-free HTTP transport implementation for runtime commit submit/finality calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitHttpTransport {
    timeout_seconds: u64,
    authorization_header: Option<String>,
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
        let mut request = format!(
            "{method} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
            endpoint.target_path, endpoint.host_header
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
        stream.write_all(request).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;

        let mut response_bytes = Vec::new();
        stream.read_to_end(&mut response_bytes).map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        Ok(response_bytes)
    }

    fn execute_https_request(
        &self,
        endpoint: ParsedHttpEndpoint,
        request: &[u8],
    ) -> Result<Vec<u8>, KolmeRuntimeCommitProviderError> {
        let connect_target = format!("{}:{}", endpoint.host, endpoint.port);
        let mut command = Command::new("openssl");
        command
            .arg("s_client")
            .arg("-quiet")
            .arg("-verify_return_error")
            .arg("-servername")
            .arg(endpoint.host.as_str())
            .arg("-connect")
            .arg(connect_target.as_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let configured_ca_file =
            resolve_kolme_tls_ca_file_env_result_contract(std::env::var("KAMN_KOLME_TLS_CA_FILE"))
                .map_err(|error| match error {
                    KamnKolmeTlsPolicyError::Unavailable { reason } => {
                        KolmeRuntimeCommitProviderError::Unavailable { reason }
                    }
                })?;
        if let Some(ca_file) = configured_ca_file {
            command.arg("-CAfile").arg(ca_file);
        }

        let mut child =
            command
                .spawn()
                .map_err(|error| KolmeRuntimeCommitProviderError::Unavailable {
                    reason: format!("tls command spawn failed: {error}"),
                })?;
        {
            let mut stdin =
                child
                    .stdin
                    .take()
                    .ok_or_else(|| KolmeRuntimeCommitProviderError::Unavailable {
                        reason: "tls command stdin unavailable".to_owned(),
                    })?;
            stdin.write_all(request).map_err(|error| {
                KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
            })?;
        }

        let timeout = Duration::from_secs(self.timeout_seconds);
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if started.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(KolmeRuntimeCommitProviderError::Timeout);
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => {
                    return Err(KolmeRuntimeCommitProviderError::Unavailable {
                        reason: format!("tls command wait failed: {error}"),
                    });
                }
            }
        }

        let output = child.wait_with_output().map_err(|error| {
            KolmeRuntimeCommitProviderError::from(classify_kolme_transport_io_error(&error))
        })?;
        let looks_like_http_response = output.stdout.starts_with(b"HTTP/1.")
            && output.stdout.windows(4).any(|window| window == b"\r\n\r\n");
        if !output.status.success() && !looks_like_http_response {
            return Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: classify_kolme_tls_failure_reason(
                    String::from_utf8_lossy(&output.stderr).as_ref(),
                ),
            });
        }
        if !is_kolme_valid_http_response_bytes_input_contract(output.stdout.as_slice()) {
            return Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: "tls response body is empty".to_owned(),
            });
        }
        Ok(output.stdout)
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
