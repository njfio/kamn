use kamn_core::{
    KolmeCommitReceiptFinality, KolmeRuntimeCommitBlockFallbackReconciler,
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitHttpTransport,
    KolmeRuntimeCommitProviderError,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

type BlockLookupCalls = Rc<RefCell<Vec<(String, String, u64)>>>;

#[derive(Debug, Clone)]
struct RecordingBlockLookupTransport {
    calls: BlockLookupCalls,
    responses: Rc<RefCell<VecDeque<Result<String, KolmeRuntimeCommitProviderError>>>>,
}

impl RecordingBlockLookupTransport {
    fn with_responses(
        responses: Vec<Result<String, KolmeRuntimeCommitProviderError>>,
    ) -> (Self, BlockLookupCalls) {
        let calls = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                responses: Rc::new(RefCell::new(VecDeque::from(responses))),
            },
            calls,
        )
    }
}

impl KolmeRuntimeCommitBlockFallbackTransport for RecordingBlockLookupTransport {
    fn fetch_block_by_height(
        &mut self,
        base_url: &str,
        block_path_template: &str,
        height: u64,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        self.calls
            .borrow_mut()
            .push((base_url.to_owned(), block_path_template.to_owned(), height));
        self.responses.borrow_mut().pop_front().unwrap_or_else(|| {
            Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "no queued block lookup response".to_owned(),
            })
        })
    }
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");

    loop {
        let read_count = stream
            .read(&mut chunk)
            .expect("request bytes should be readable");
        if read_count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_count]);

        if header_end.is_none() {
            header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|pos| pos + 4);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&buffer[..end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        if name.eq_ignore_ascii_case("Content-Length") {
                            return value.trim().parse::<usize>().ok();
                        }
                        None
                    })
                    .unwrap_or(0);
                expected_total = Some(end + content_length);
            }
        }

        if let Some(total) = expected_total {
            if buffer.len() >= total {
                break;
            }
        }
    }

    String::from_utf8(buffer).expect("request should be valid utf-8")
}

fn spawn_block_lookup_server(expected_paths_and_bodies: Vec<(String, String)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        for (expected_path, response_body) in expected_paths_and_bodies {
            let (mut stream, _) = listener.accept().expect("connection should be accepted");
            let request = read_http_request(&mut stream);
            assert!(
                request.contains(format!("GET {expected_path} HTTP/1.1").as_str()),
                "expected path {expected_path} in request, got: {request}"
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream
                .write_all(response.as_bytes())
                .expect("response should write");
        }
    });
    format!("http://{addr}")
}

#[test]
fn unit_block_fallback_reconciler_rejects_stale_lookup_window() {
    let (transport, _calls) = RecordingBlockLookupTransport::with_responses(vec![]);
    let mut reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        3,
        transport,
    )
    .expect("reconciler should build");

    assert_eq!(
        reconciler.reconcile_txhash("ab12cd34", 40, 44),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "block fallback window exceeds max lookups: from_height=40 latest_height=44 max_lookups=3"
                .to_owned(),
        })
    );
}

#[test]
fn functional_block_fallback_reconciles_final_receipt_from_block_lookup() {
    let responses = vec![
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":40,\"tx_hashes\":\"ff00\"}".to_owned()),
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":41,\"tx_hashes\":\"00aa\"}".to_owned()),
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":42,\"tx_hashes\":\"ab12cd34,ffee\"}".to_owned()),
    ];
    let (transport, calls) = RecordingBlockLookupTransport::with_responses(responses);
    let mut reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        6,
        transport,
    )
    .expect("reconciler should build");

    let receipt = reconciler
        .reconcile_txhash("ab12cd34", 40, 42)
        .expect("fallback should converge to final receipt");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34:h42");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);

    let observed_heights = calls
        .borrow()
        .iter()
        .map(|(_, _, height)| *height)
        .collect::<Vec<_>>();
    assert_eq!(observed_heights, vec![40, 41, 42]);
}

#[test]
fn functional_block_fallback_reconciles_failed_receipt_from_block_lookup() {
    let responses = vec![Ok(
        "{\"provider\":\"kolme-fork-local\",\"block_height\":50,\"failed_tx_hashes\":\"ab12cd34\"}"
            .to_owned(),
    )];
    let (transport, _calls) = RecordingBlockLookupTransport::with_responses(responses);
    let mut reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        2,
        transport,
    )
    .expect("reconciler should build");

    let receipt = reconciler
        .reconcile_txhash("ab12cd34", 50, 50)
        .expect("fallback should converge to failed receipt");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Failed);
}

#[test]
fn integration_http_transport_block_fallback_converges_with_mock_block_api() {
    let base_url = spawn_block_lookup_server(vec![
        (
            "/block/70".to_owned(),
            "{\"provider\":\"kolme-fork-local\",\"block_height\":70,\"tx_hashes\":\"deadbeef\"}"
                .to_owned(),
        ),
        (
            "/block/71".to_owned(),
            "{\"provider\":\"kolme-fork-local\",\"block_height\":71,\"tx_hashes\":\"ab12cd34\"}"
                .to_owned(),
        ),
    ]);
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        base_url.as_str(),
        "/block/{height}",
        "kolme-fork-local",
        3,
        transport,
    )
    .expect("reconciler should build");

    let receipt = reconciler
        .reconcile_txhash("ab12cd34", 70, 71)
        .expect("fallback should find txhash in block window");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34:h71");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_block_fallback_rejects_response_height_mismatch_fail_closed() {
    // Regression: #1464
    let responses = vec![Ok(
        "{\"provider\":\"kolme-fork-local\",\"block_height\":99,\"tx_hashes\":\"ab12cd34\"}"
            .to_owned(),
    )];
    let (transport, _calls) = RecordingBlockLookupTransport::with_responses(responses);
    let mut reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        2,
        transport,
    )
    .expect("reconciler should build");

    assert_eq!(
        reconciler.reconcile_txhash("ab12cd34", 42, 42),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "block fallback response height mismatch: expected 42 observed 99".to_owned(),
        })
    );
}
