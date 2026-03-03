use super::{
    parse_unmasked_text_frame_payload, read_response_bytes, resolve_request_timeout_seconds,
    service_public_key_for_private_key, service_signature_for_state_hash_with_private_key,
    service_verify_signature_with_public_key, write_and_flush_request, SdkError,
    MAX_SERVICE_RESPONSE_BYTES, REQUEST_TIMEOUT_SECONDS_ENV,
    SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED, SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED,
};
use crate::AgentDid;
use std::collections::VecDeque;
use std::ffi::OsString;
use std::io::{ErrorKind, Read, Write};
use std::sync::{Mutex, OnceLock};

enum ReadStep {
    Bytes(Vec<u8>),
    Error(ErrorKind),
    RepeatByte(u8),
    Eof,
}

struct ScriptedReader {
    steps: VecDeque<ReadStep>,
}

impl ScriptedReader {
    fn new(steps: impl IntoIterator<Item = ReadStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }
}

impl Read for ScriptedReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let Some(step) = self.steps.front_mut() else {
            return Ok(0);
        };
        match step {
            ReadStep::Bytes(bytes) => {
                let read_count = buffer.len().min(bytes.len());
                buffer[..read_count].copy_from_slice(&bytes[..read_count]);
                bytes.drain(..read_count);
                if bytes.is_empty() {
                    let _ = self.steps.pop_front();
                }
                Ok(read_count)
            }
            ReadStep::Error(kind) => {
                let kind = *kind;
                let _ = self.steps.pop_front();
                Err(std::io::Error::from(kind))
            }
            ReadStep::RepeatByte(byte) => {
                buffer[0] = *byte;
                Ok(1)
            }
            ReadStep::Eof => {
                let _ = self.steps.pop_front();
                Ok(0)
            }
        }
    }
}

struct RecordingWriter {
    bytes: Vec<u8>,
    flush_calls: u64,
    fail_flush: bool,
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    fn set_utf8(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        // SAFETY: tests serialize env writes with test-local guard lifetime.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }

    fn unset(key: &'static str) -> Self {
        let previous = std::env::var_os(key);
        // SAFETY: tests serialize env writes with test-local guard lifetime.
        unsafe { std::env::remove_var(key) };
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.previous.as_ref() {
            Some(previous) => {
                // SAFETY: restoring process env value captured at guard construction.
                unsafe { std::env::set_var(self.key, previous) };
            }
            None => {
                // SAFETY: restoring process env to prior absent state.
                unsafe { std::env::remove_var(self.key) };
            }
        }
    }
}

fn timeout_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

impl RecordingWriter {
    fn with_flush_failure(fail_flush: bool) -> Self {
        Self {
            bytes: Vec::new(),
            flush_calls: 0,
            fail_flush,
        }
    }
}

impl Write for RecordingWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_calls = self.flush_calls.saturating_add(1);
        if self.fail_flush {
            return Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "synthetic flush failure",
            ));
        }
        Ok(())
    }
}

#[test]
fn unit_write_and_flush_request_invokes_flush_on_success_path() {
    let mut writer = RecordingWriter::with_flush_failure(false);
    let payload = b"GET /healthz HTTP/1.1\r\n\r\n";
    write_and_flush_request(&mut writer, payload, "failed to write service request")
        .expect("write and flush should succeed");
    assert_eq!(writer.bytes, payload);
    assert_eq!(
        writer.flush_calls, 1,
        "flush should be invoked exactly once"
    );
}

#[test]
fn regression_issue_6208_request_timeout_defaults_when_env_missing() {
    let _lock = timeout_env_lock().lock().expect("lock poisoned");
    let _guard = EnvVarGuard::unset(REQUEST_TIMEOUT_SECONDS_ENV);
    assert_eq!(resolve_request_timeout_seconds(), Ok(2));
}

#[test]
fn regression_issue_6208_request_timeout_accepts_configured_positive_value() {
    let _lock = timeout_env_lock().lock().expect("lock poisoned");
    let _guard = EnvVarGuard::set_utf8(REQUEST_TIMEOUT_SECONDS_ENV, "7");
    assert_eq!(resolve_request_timeout_seconds(), Ok(7));
}

#[test]
fn regression_issue_6208_request_timeout_rejects_zero_or_non_numeric_values() {
    let _lock = timeout_env_lock().lock().expect("lock poisoned");
    let _guard = EnvVarGuard::set_utf8(REQUEST_TIMEOUT_SECONDS_ENV, "0");
    assert_eq!(
        resolve_request_timeout_seconds(),
        Err(SdkError::InvalidInput {
            field: "service.request_timeout_seconds",
            reason: "must be greater than zero",
        })
    );

    let _guard = EnvVarGuard::set_utf8(REQUEST_TIMEOUT_SECONDS_ENV, "fast");
    assert_eq!(
        resolve_request_timeout_seconds(),
        Err(SdkError::InvalidInput {
            field: "service.request_timeout_seconds",
            reason: "must be valid integer seconds",
        })
    );
}

#[test]
fn regression_write_and_flush_request_propagates_flush_failure() {
    // Regression: #5953
    let mut writer = RecordingWriter::with_flush_failure(true);
    let error = write_and_flush_request(&mut writer, b"{}", "failed to write service request")
        .expect_err("flush failure should fail closed");
    assert_eq!(
        writer.flush_calls, 1,
        "flush failure path must invoke flush"
    );
    assert_eq!(
        error,
        SdkError::TransportFailure("failed to write service request")
    );
}

#[test]
fn regression_read_response_bytes_allows_partial_payload_before_unexpected_eof() {
    // Regression: #5953
    let payload = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
    let mut reader = ScriptedReader::new([
        ReadStep::Bytes(payload.to_vec()),
        ReadStep::Error(ErrorKind::UnexpectedEof),
        ReadStep::Eof,
    ]);
    let response = read_response_bytes(&mut reader)
        .expect("partial payload should be preserved across unexpected eof");
    assert_eq!(response, payload);
}

#[test]
fn regression_read_response_bytes_rejects_unexpected_eof_without_payload() {
    // Regression: #5953
    let mut reader = ScriptedReader::new([ReadStep::Error(ErrorKind::UnexpectedEof)]);
    let error = read_response_bytes(&mut reader)
        .expect_err("unexpected eof without payload should fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure("failed to read service response payload")
    );
}

#[test]
fn regression_read_response_bytes_fails_closed_when_iteration_budget_exceeded() {
    // Regression: #5953
    let mut reader = ScriptedReader::new([ReadStep::RepeatByte(b'x')]);
    let error = read_response_bytes(&mut reader)
        .expect_err("pathological single-byte stream should exceed iteration budget");
    assert_eq!(
        error,
        SdkError::TransportFailure(SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED)
    );
}

#[test]
fn unit_read_response_bytes_accepts_payload_at_exact_size_limit() {
    let payload = vec![b'a'; MAX_SERVICE_RESPONSE_BYTES];
    let mut reader = ScriptedReader::new([ReadStep::Bytes(payload.clone()), ReadStep::Eof]);
    let response =
        read_response_bytes(&mut reader).expect("exact limit payload should be accepted");
    assert_eq!(response.len(), MAX_SERVICE_RESPONSE_BYTES);
    assert_eq!(response, payload);
}

#[test]
fn regression_read_response_bytes_rejects_payload_exceeding_size_limit() {
    // Regression: #5953
    let payload = vec![b'b'; MAX_SERVICE_RESPONSE_BYTES.saturating_add(1)];
    let mut reader = ScriptedReader::new([ReadStep::Bytes(payload), ReadStep::Eof]);
    let error = read_response_bytes(&mut reader)
        .expect_err("payloads exceeding size limit should fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure(SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED)
    );
}

fn websocket_text_frame(payload: &[u8], length_mode: u8) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(0x81);
    match length_mode {
        125 => {
            frame.push(payload.len() as u8);
        }
        126 => {
            frame.push(126);
            frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        }
        127 => {
            frame.push(127);
            frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
        }
        _ => panic!("unsupported test length mode"),
    }
    frame.extend_from_slice(payload);
    frame
}

#[test]
fn unit_parse_websocket_frame_payload_accepts_16bit_extended_length() {
    let payload = vec![b'a'; 130];
    let frame = websocket_text_frame(payload.as_slice(), 126);
    let parsed = parse_unmasked_text_frame_payload(frame.as_slice())
        .expect("16-bit extended websocket payload should parse");
    assert_eq!(parsed, payload.as_slice());
}

#[test]
fn unit_parse_websocket_frame_payload_accepts_64bit_extended_length() {
    let payload = vec![b'b'; 130];
    let frame = websocket_text_frame(payload.as_slice(), 127);
    let parsed = parse_unmasked_text_frame_payload(frame.as_slice())
        .expect("64-bit extended websocket payload should parse");
    assert_eq!(parsed, payload.as_slice());
}

#[test]
fn regression_parse_websocket_frame_payload_rejects_truncated_extended_header() {
    // Regression: #6190
    let frame = vec![0x81, 126, 0x00];
    let error = parse_unmasked_text_frame_payload(frame.as_slice())
        .expect_err("truncated extended-length header must fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure("service websocket response frame payload truncated")
    );
}

const TEST_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

#[test]
fn unit_service_public_key_for_private_key_derives_compressed_hex_key() {
    let public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
        .expect("valid private key should derive signer public key");
    assert_eq!(public_key.len(), 66);
    assert!(
        public_key.starts_with("02") || public_key.starts_with("03"),
        "compressed secp256k1 key should start with 02 or 03"
    );
}

#[test]
fn regression_service_public_key_for_private_key_rejects_invalid_private_key_hex() {
    // Regression: #5977
    let error = service_public_key_for_private_key("not-a-private-key")
        .expect_err("invalid key material must fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "must be valid secp256k1 private key hex",
        }
    );
}

#[test]
fn unit_service_verify_signature_with_public_key_accepts_valid_signature() {
    let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
    let state_hash = "service-api:kamn-sdk:1";
    let body = r#"{"message":"hello"}"#;
    let signature = service_signature_for_state_hash_with_private_key(
        &sender_did,
        7,
        state_hash,
        body,
        TEST_PRIVATE_KEY_HEX,
    )
    .expect("signature should be produced");
    let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
        .expect("public key should be derived");
    service_verify_signature_with_public_key(
        &sender_did,
        7,
        state_hash,
        body,
        signature.as_str(),
        signer_public_key.as_str(),
    )
    .expect("valid signature should verify");
}

#[test]
fn regression_service_verify_signature_with_public_key_rejects_invalid_public_key_hex() {
    // Regression: #5977
    let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
    let signature = service_signature_for_state_hash_with_private_key(
        &sender_did,
        8,
        "service-api:kamn-sdk:1",
        "{}",
        TEST_PRIVATE_KEY_HEX,
    )
    .expect("signature should be produced");
    let error = service_verify_signature_with_public_key(
        &sender_did,
        8,
        "service-api:kamn-sdk:1",
        "{}",
        signature.as_str(),
        "invalid-public-key",
    )
    .expect_err("invalid public key should fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "service.request_auth.expected_public_key",
            reason: "must be valid compressed secp256k1 public key hex",
        }
    );
}

#[test]
fn regression_service_verify_signature_with_public_key_rejects_empty_state_hash() {
    // Regression: #5977
    let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
    let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
        .expect("public key should be derived");
    let signature = service_signature_for_state_hash_with_private_key(
        &sender_did,
        9,
        "service-api:kamn-sdk:1",
        "{}",
        TEST_PRIVATE_KEY_HEX,
    )
    .expect("signature should be produced");
    let error = service_verify_signature_with_public_key(
        &sender_did,
        9,
        "",
        "{}",
        signature.as_str(),
        signer_public_key.as_str(),
    )
    .expect_err("empty state hash should fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "service.request_auth.state_hash",
            reason: "must not be empty",
        }
    );
}

#[test]
fn regression_service_verify_signature_with_public_key_rejects_empty_signature() {
    // Regression: #5977
    let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
    let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
        .expect("public key should be derived");
    let error = service_verify_signature_with_public_key(
        &sender_did,
        10,
        "service-api:kamn-sdk:1",
        "{}",
        "",
        signer_public_key.as_str(),
    )
    .expect_err("empty signature should fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "must not be empty",
        }
    );
}

#[test]
fn regression_service_verify_signature_with_public_key_rejects_non_positive_nonce() {
    // Regression: #5977
    let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
    let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
        .expect("public key should be derived");
    let signature = service_signature_for_state_hash_with_private_key(
        &sender_did,
        1,
        "service-api:kamn-sdk:1",
        "{}",
        TEST_PRIVATE_KEY_HEX,
    )
    .expect("signature should be produced");
    let error = service_verify_signature_with_public_key(
        &sender_did,
        0,
        "service-api:kamn-sdk:1",
        "{}",
        signature.as_str(),
        signer_public_key.as_str(),
    )
    .expect_err("non-positive nonce should fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "service.request_auth.nonce",
            reason: "must be greater than zero",
        }
    );
}

#[test]
fn regression_service_verify_signature_with_public_key_rejects_tampered_signature() {
    // Regression: #5977
    let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
    let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
        .expect("public key should be derived");
    let mut signature = service_signature_for_state_hash_with_private_key(
        &sender_did,
        11,
        "service-api:kamn-sdk:1",
        "{}",
        TEST_PRIVATE_KEY_HEX,
    )
    .expect("signature should be produced");
    signature.push('f');
    let error = service_verify_signature_with_public_key(
        &sender_did,
        11,
        "service-api:kamn-sdk:1",
        "{}",
        signature.as_str(),
        signer_public_key.as_str(),
    )
    .expect_err("tampered signature should fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "failed cryptographic signature verification",
        }
    );
}
