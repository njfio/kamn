use super::super::*;
use super::http_transport_support::{render_http_request};

pub(crate) const TEST_SERVICE_API_TLS_CERT_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIDCTCCAfGgAwIBAgIUX9dYtx2K5dX0X33CQvg4re7nVwwwDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDIxNTEzMDkwNFoXDTI2MDIx
NjEzMDkwNFowFDESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEApfGNzxPiL+e4z6Pok8RT5RkZE631O/Pg7VBgN4xnCjTz
xwjDihOJSCBl1wYM09xeFUHE6JjTO2ABHdmtXJxXWaAygWRUvdYOBbf1c8ObkanC
+0f0xzUn8rxYyDo8PknR9QR32dCVG5LM5XrIw08TQPAZxEdOEKPkgDqeCWRGsWO/
YbaziAHXNsNShvYucAlHxzfhXnhRhVKrdVyZ0G7wZZAZoMgSC15lWDWw1JxVbBqr
0ui8eajKEDg8NZz9mw0VEYGCJGacgn/Y7+YQviEKNL+2yj57LbGsFrXRfSczpNxV
JmgXChRy5849aLJsatm1NSAhYmFamX7d+7EErKPwhQIDAQABo1MwUTAdBgNVHQ4E
FgQU/EbABKdaVJZGhOBJ2/WodsjxNJcwHwYDVR0jBBgwFoAU/EbABKdaVJZGhOBJ
2/WodsjxNJcwDwYDVR0TAQH/BAUwAwEB/zANBgkqhkiG9w0BAQsFAAOCAQEAcYPD
j0Me1W3oQkgz9yGT75IYrM6bdSJRQt+vIKQI5AAVIqX5IoGfjP/zJFner96T0i/7
rPVinMFmyYTXYr/qqbQ9jdLt9FS+l0eIqN9oCmHC6Anhn9/FORZzBsIBQDPkZxXk
G5QUhQ/joTqTdUaQcrKh4UeRA1LJtlAnFnYc3CeQdKQQqB4W5JeZSdsU1E0FU5wl
fE7ucg85yIEn33V6aCexCfHhDh2TnLo25awqoyNCbFhu7DLnbnyOeKSB5lI3TdvK
ag0XPq+nohTyUBXw+XUR2PnYXOEGZxBQdhvyQO0ib/y2dcODuYbXkQDq+f0UuBbn
R/+8zPGgzivZEPa01Q==
-----END CERTIFICATE-----
";

pub(crate) const TEST_SERVICE_API_TLS_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCl8Y3PE+Iv57jP
o+iTxFPlGRkTrfU78+DtUGA3jGcKNPPHCMOKE4lIIGXXBgzT3F4VQcTomNM7YAEd
2a1cnFdZoDKBZFS91g4Ft/Vzw5uRqcL7R/THNSfyvFjIOjw+SdH1BHfZ0JUbkszl
esjDTxNA8BnER04Qo+SAOp4JZEaxY79htrOIAdc2w1KG9i5wCUfHN+FeeFGFUqt1
XJnQbvBlkBmgyBILXmVYNbDUnFVsGqvS6Lx5qMoQODw1nP2bDRURgYIkZpyCf9jv
5hC+IQo0v7bKPnstsawWtdF9JzOk3FUmaBcKFHLnzj1osmxq2bU1ICFiYVqZft37
sQSso/CFAgMBAAECggEACrFhAOn4FjwpRX/7WaI6AbY3TnRULBPP95rJSGsMrLSy
zK185CXUH8iup0dlhjVZ/qapSI+odNf/2muPZztPyZ+wAXR0nXLwnl+3Okltedpl
jQma9UcwlsyaL/TIsv7Qv6gVDP0KzqcL+vGJhERRKksObf5mQl49OCIO0u4aPA3l
0Y0h9WVtvydyhztQCFfVkkZNgiAY2WSI73xO72RFU0ZKnwc9ZVvona7yTKJIpV+i
3k0N/27kfc21UUtXJ7Nv5b07MIH8vkx+c1FX63vAPkyBdfXguNG/Yn/uVGqq2fbZ
xypp3JIRW3b2Heo/Ox02791gRuWJcpEmU369E4fq9QKBgQDT2T5m5HfHcA4Zpjk5
HtPvdINWntwkSZw8E/41LVY0PptOqM3yWDSb0TLQCoefhtPWm571RvSdevU+NpyB
jnzx+8gEXAeC1D6TKYmucO0wv7A5ZqC1WzLO7LKG4DuJbANs9PApuqPkzPAGegey
NkVOTRWO7ggLzmPxYFN8leW+fwKBgQDIhyOkchw1cl+GMDibnrB4Ynljvlxn0tDo
A4N3oSTv1Az8mO7DGJ+S/mY8aYmw4ogbPIGXZkxlhie0pS7kfttzswbY6TePgml6
pbLvfzv9OGUKLp0QhmNzfNygP6A8pIb2vbuEYJl6boE/jEIG7c8E0VmsUqe60Aoz
EcDLDtzW+wKBgQC5Hnj9CF/ykuR/XVVbqKih8jpikubjfr9bcE0OwtM1TBACqFdu
kc1G64NvcAQbToIGYm6A/sP6aNusxaP1QkHEYrPhu1mE5VrY1c9N87gQhTDEt/1u
/IZlc0h9u6vK5ewIZfEHReS5pquHvVLEU9A0H//aqf2182A6KGZL0+CymQKBgGsd
xSxSyD7EmcJUf+ihHCMydyWQykurkWxedBuzOMfjvgwwpVoSDSu4OWSL+8FBQPNL
nu4A905EG3GjyyjDmvZy63VzHvrJ7w5U9QB6NtFNDqwhukTZhMZsLG5tjmrWeEHV
mBVehJ2h6ejIQ3zwC2XHbt9eR7rC5q/hC9tsVQuBAoGAG8WncvZ15/VwncKQwz3G
bgoNsx0W5SO8NNfecDRVJLsCCuy5M9s5vn/u1Xz7l9pA0vCup9l6v96hQJTQBEQ6
urk/MQl1UlrSRdDK2gu40MToc8X5ig0dVDVG5QhPl7YmUu9G2EAL3WZTpJXsRh22
VpYUFFjotXCdBIUnUQ51PGg=
-----END PRIVATE KEY-----
";

#[derive(Debug)]
pub(crate) struct TestSkipServerVerification(Arc<rustls::crypto::CryptoProvider>);

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
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.0.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

pub(crate) fn send_https_request_with_headers(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
    root_cert_pem: &str,
) -> String {
    send_https_request_with_headers_raw(addr, method, path, body, headers, root_cert_pem)
}

pub(crate) fn send_https_request_with_headers_raw(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
    _root_cert_pem: &str,
) -> String {
    let client_config = build_tls_client_config();
    let server_name = rustls::pki_types::ServerName::try_from("localhost".to_owned())
        .expect("server name should parse");
    let connection = rustls::ClientConnection::new(Arc::new(client_config), server_name)
        .expect("tls client connection should initialize");
    let tcp_stream = configure_tls_stream(addr);
    let mut stream = rustls::StreamOwned::new(connection, tcp_stream);
    let request = render_http_request("localhost", method, path, body, headers);
    stream.write_all(request.as_bytes()).expect("tls request should write");
    stream.flush().expect("tls request should flush");
    read_tls_response(&mut stream)
}

pub(crate) fn write_test_service_api_tls_materials() -> (String, String) {
    let base = tls_temp_dir();
    fs::create_dir_all(&base).expect("temporary tls directory should be created");
    let cert_path = base.join("server-cert.pem");
    let key_path = base.join("server-key.pem");
    fs::write(&cert_path, TEST_SERVICE_API_TLS_CERT_PEM.as_bytes()).expect("test cert should write");
    fs::write(&key_path, TEST_SERVICE_API_TLS_KEY_PEM.as_bytes()).expect("test key should write");
    (cert_path.to_string_lossy().to_string(), key_path.to_string_lossy().to_string())
}

fn build_tls_client_config() -> rustls::ClientConfig {
    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(TestSkipServerVerification::new())
        .with_no_client_auth()
}

fn configure_tls_stream(addr: &str) -> TcpStream {
    let stream = TcpStream::connect(addr).expect("tls endpoint should accept connections");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("tls read timeout should be configurable");
    stream
}

fn read_tls_response(stream: &mut rustls::StreamOwned<rustls::ClientConnection, TcpStream>) -> String {
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(count) => response.push_str(std::str::from_utf8(&chunk[..count]).expect("response must be utf-8")),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => break,
            Err(error) => panic!("tls response should be readable: {error}"),
        }
    }
    response
}

fn tls_temp_dir() -> std::path::PathBuf {
    let entropy = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock should be monotonic").as_nanos();
    std::env::temp_dir().join(format!("kamn-node-service-api-tls-{}-{entropy}", std::process::id()))
}
