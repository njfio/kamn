use super::*;

const HTTPS_RESPONSE_BODY: &str =
    "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:https\nfinality=final\n";

pub(crate) fn with_trusted_https_server(run: impl FnOnce(HttpsSingleRequestServer)) {
    let (_guard, _env_guard) = tls_env_scope(None);
    let server = spawn_https_single_request_server(200, HTTPS_RESPONSE_BODY);
    let ca_cert_path = server
        .ca_cert_path
        .to_str()
        .expect("temporary cert path should be valid utf-8")
        .to_owned();
    let _ca_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, Some(ca_cert_path.as_str()));
    run(server);
}

pub(crate) fn with_untrusted_https_server(run: impl FnOnce(HttpsSingleRequestServer)) {
    let (_guard, _env_guard) = tls_env_scope(None);
    run(spawn_https_single_request_server(200, HTTPS_RESPONSE_BODY));
}

pub(crate) fn with_tls_env_none<T>(run: impl FnOnce() -> T) -> T {
    let (_guard, _env_guard) = tls_env_scope(None);
    run()
}

fn tls_env_scope(value: Option<&str>) -> (std::sync::MutexGuard<'static, ()>, EnvVarGuard) {
    let guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, value);
    (guard, env_guard)
}

pub(crate) fn https_provider(
    base_url: &str,
    timeout_seconds: u64,
) -> KolmeRuntimeCommitLiveProvider<KolmeRuntimeCommitHttpTransport> {
    let transport =
        KolmeRuntimeCommitHttpTransport::new(timeout_seconds).expect("transport should build");
    KolmeRuntimeCommitLiveProvider::new(base_url, "/broadcast/runtime-commit", transport)
        .expect("provider should build")
}

pub(crate) fn assert_submitted_https_receipt(
    outcome: KolmeRuntimeCommitProviderOutcome,
    commit_id: &str,
) {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, commit_id);
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

pub(crate) fn spawn_plain_http_over_tls_socket(sleep: Duration) -> std::net::SocketAddr {
    spawn_tls_socket_listener(sleep)
}

fn spawn_tls_socket_listener(sleep: Duration) -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        if sleep.is_zero() {
            let _ = stream.read(&mut [0_u8; 64]);
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            return;
        }
        thread::sleep(sleep);
    });
    addr
}

pub(crate) fn transport_source() -> &'static str {
    include_str!("../../../src/kolme_runtime_commit/http_transport.rs")
}

pub(crate) fn tls_adr_source() -> &'static str {
    include_str!("../../../../../docs/architecture/adr-kamn-core-live-tls-transport.md")
}
