use super::super::*;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};

#[derive(Debug)]
struct TestSkipServerVerification(Arc<rustls::crypto::CryptoProvider>);

impl TestSkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl rustls::client::danger::ServerCertVerifier for TestSkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

struct ObservabilityEndpointTlsModeOverrideGuard;

impl Drop for ObservabilityEndpointTlsModeOverrideGuard {
    fn drop(&mut self) {
        set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(None);
    }
}

fn build_client_stream(
    addr: &str,
) -> Result<rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>, String> {
    let client_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(TestSkipServerVerification::new())
        .with_no_client_auth();
    let server_name = rustls::pki_types::ServerName::try_from("localhost".to_owned())
        .map_err(|error| format!("server name should parse: {error}"))?;
    let connection = rustls::ClientConnection::new(Arc::new(client_config), server_name)
        .map_err(|error| format!("tls client connection should initialize: {error}"))?;
    let tcp_stream = std::net::TcpStream::connect(addr)
        .map_err(|error| format!("tls connect should succeed: {error}"))?;
    tcp_stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("tls read timeout should be configurable: {error}"))?;
    Ok(rustls::StreamOwned::new(connection, tcp_stream))
}

fn read_tls_response(
    stream: &mut rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>,
) -> Result<String, String> {
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return Ok(response),
            Ok(read_count) => response.push_str(
                std::str::from_utf8(&chunk[..read_count])
                    .map_err(|error| format!("response must be utf-8: {error}"))?,
            ),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return Ok(response);
            }
            Err(error) => return Err(format!("tls response should be readable: {error}")),
        }
    }
}

pub(in super::super) fn send_https_get(addr: &str, path: &str) -> String {
    try_send_https_get(addr, path).expect("tls request should succeed")
}

pub(in super::super) fn try_send_https_get(addr: &str, path: &str) -> Result<String, String> {
    let mut stream = build_client_stream(addr)?;
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("tls request should write: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("tls request should flush: {error}"))?;
    read_tls_response(&mut stream)
}

pub(in super::super) fn wait_for_https_endpoint_ready(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if try_send_https_get(addr, "/readyz")
            .map(|response| response.contains("HTTP/1.1"))
            .unwrap_or(false)
        {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("tls endpoint did not become ready within timeout");
}

pub(in super::super) fn set_tls_mode_override_for_current_thread(
    mode: ObservabilityEndpointTlsModeOverride,
) -> impl Drop {
    set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(Some(mode));
    ObservabilityEndpointTlsModeOverrideGuard
}

pub(in super::super) fn observability_tls_temp_path(label: &str) -> String {
    let entropy = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir()
        .join(format!(
            "kamn-observability-tls-{label}-{}-{entropy}.pem",
            std::process::id()
        ))
        .to_string_lossy()
        .to_string()
}
