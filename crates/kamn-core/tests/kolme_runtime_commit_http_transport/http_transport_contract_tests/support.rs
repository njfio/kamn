use super::*;

pub(crate) fn submitted_response(commit_id: &str) -> String {
    format!("status=submitted\nprovider=kolme-local\ncommit_id={commit_id}\nfinality=final\n")
}

pub(crate) fn provider(base_url: &str, path: &str, timeout_seconds: u64) -> KolmeRuntimeCommitLiveProvider<KolmeRuntimeCommitHttpTransport> {
    let transport = KolmeRuntimeCommitHttpTransport::new(timeout_seconds)
        .expect("transport should build");
    KolmeRuntimeCommitLiveProvider::new(base_url, path, transport)
        .expect("provider should build")
}

pub(crate) fn checker(base_url: &str, path: &str) -> KolmeRuntimeCommitFinalityChecker<KolmeRuntimeCommitHttpTransport> {
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    KolmeRuntimeCommitFinalityChecker::new(base_url, path, transport)
        .expect("checker should build")
}

pub(crate) fn assert_submitted_receipt(
    outcome: KolmeRuntimeCommitProviderOutcome,
    provider_name: &str,
    commit_id: &str,
) {
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, provider_name);
            assert_eq!(receipt.commit_id, commit_id);
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

pub(crate) fn assert_finality_receipt(
    receipt: KolmeCommitReceiptFinality,
    expected: KolmeCommitReceiptFinality,
) {
    assert_eq!(receipt, expected);
}

pub(crate) fn timeout_listener_url(sleep: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("connection should be accepted");
        thread::sleep(sleep);
    });
    format!("http://{addr}")
}

pub(crate) fn status_server(base_url_body: String, matcher: impl Fn(String) + Send + 'static) -> String {
    spawn_single_request_server(base_url_body, "HTTP/1.1 200 OK", matcher)
}
