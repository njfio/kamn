use kamn_core::{
    KolmeCommitReceiptFinality, KolmeRuntimeCommitBlockFallbackReconciler,
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitForkFinalityResolver,
    KolmeRuntimeCommitNotificationsConnection, KolmeRuntimeCommitNotificationsConnector,
    KolmeRuntimeCommitNotificationsConsumer, KolmeRuntimeCommitProviderError,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Clone)]
enum MockStep {
    Message(String),
    Closed,
}

#[derive(Clone)]
struct MockConnection {
    steps: Rc<RefCell<VecDeque<MockStep>>>,
}

impl MockConnection {
    fn new(steps: Vec<MockStep>) -> Self {
        Self {
            steps: Rc::new(RefCell::new(VecDeque::from(steps))),
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnection for MockConnection {
    fn read_text_message(&mut self) -> Result<Option<String>, KolmeRuntimeCommitProviderError> {
        let Some(step) = self.steps.borrow_mut().pop_front() else {
            return Ok(None);
        };
        match step {
            MockStep::Message(payload) => Ok(Some(payload)),
            MockStep::Closed => Ok(None),
        }
    }
}

struct MockConnector {
    connections: VecDeque<Result<MockConnection, KolmeRuntimeCommitProviderError>>,
}

impl MockConnector {
    fn new(connections: Vec<Result<MockConnection, KolmeRuntimeCommitProviderError>>) -> Self {
        Self {
            connections: VecDeque::from(connections),
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnector for MockConnector {
    type Connection = MockConnection;

    fn connect(
        &mut self,
        _notifications_url: &str,
    ) -> Result<Self::Connection, KolmeRuntimeCommitProviderError> {
        self.connections.pop_front().unwrap_or_else(|| {
            Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "no scripted connection available".to_owned(),
            })
        })
    }
}

type BlockCalls = Rc<RefCell<Vec<(String, String, u64)>>>;

#[derive(Clone)]
struct RecordingBlockFallbackTransport {
    calls: BlockCalls,
    responses: Rc<RefCell<VecDeque<Result<String, KolmeRuntimeCommitProviderError>>>>,
}

impl RecordingBlockFallbackTransport {
    fn with_responses(
        responses: Vec<Result<String, KolmeRuntimeCommitProviderError>>,
    ) -> (Self, BlockCalls) {
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

impl KolmeRuntimeCommitBlockFallbackTransport for RecordingBlockFallbackTransport {
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
                reason: "no scripted block response".to_owned(),
            })
        })
    }
}

#[test]
fn functional_fork_finality_resolver_prefers_notifications_when_txhash_matches() {
    let connector = MockConnector::new(vec![Ok(MockConnection::new(vec![MockStep::Message(
        "{\"NewBlock\":{\"block\":{\"txhash\":\"ab12cd34\"},\"height\":42}}".to_owned(),
    )]))]);
    let notifications_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        1,
        connector,
    )
    .expect("notifications consumer should build");

    let (block_transport, block_calls) = RecordingBlockFallbackTransport::with_responses(vec![]);
    let block_reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        3,
        block_transport,
    )
    .expect("block fallback reconciler should build");

    let mut resolver =
        KolmeRuntimeCommitForkFinalityResolver::new(notifications_consumer, block_reconciler);
    let receipt = resolver
        .resolve_commit_finality("kolme-commit:ab12cd34", 40, 42)
        .expect("resolver should map notifications receipt");

    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34:h42");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
    assert!(
        block_calls.borrow().is_empty(),
        "fallback block lookups should not run when notifications resolve finality"
    );
}

#[test]
fn functional_fork_finality_resolver_falls_back_when_notifications_unavailable() {
    let connector = MockConnector::new(vec![
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "dial refused".to_owned(),
        }),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "dial refused".to_owned(),
        }),
    ]);
    let notifications_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        2,
        connector,
    )
    .expect("notifications consumer should build");

    let responses =
        vec![
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":40,\"tx_hashes\":\"00aa\"}"
            .to_owned()),
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":41,\"tx_hashes\":\"ab12cd34\"}"
            .to_owned()),
    ];
    let (block_transport, block_calls) = RecordingBlockFallbackTransport::with_responses(responses);
    let block_reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        3,
        block_transport,
    )
    .expect("block fallback reconciler should build");

    let mut resolver =
        KolmeRuntimeCommitForkFinalityResolver::new(notifications_consumer, block_reconciler);
    let receipt = resolver
        .resolve_commit_finality("kolme-commit:ab12cd34", 40, 41)
        .expect("resolver should reconcile via fallback");

    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34:h41");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
    let observed_heights = block_calls
        .borrow()
        .iter()
        .map(|(_, _, height)| *height)
        .collect::<Vec<_>>();
    assert_eq!(observed_heights, vec![40, 41]);
}

#[test]
fn functional_fork_finality_resolver_uses_new_block_height_when_txhash_is_not_present() {
    let connector = MockConnector::new(vec![Ok(MockConnection::new(vec![MockStep::Message(
        "{\"NewBlock\":{\"block\":{\"message\":\"{\\\"height\\\":42}\"},\"logs\":[[]]}}".to_owned(),
    )]))]);
    let notifications_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        1,
        connector,
    )
    .expect("notifications consumer should build");

    let responses =
        vec![
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":40,\"tx_hashes\":\"00aa\"}"
            .to_owned()),
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":41,\"tx_hashes\":\"00bb\"}"
            .to_owned()),
        Ok("{\"provider\":\"kolme-fork-local\",\"block_height\":42,\"tx_hashes\":\"ab12cd34\"}"
            .to_owned()),
    ];
    let (block_transport, block_calls) = RecordingBlockFallbackTransport::with_responses(responses);
    let block_reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        10,
        block_transport,
    )
    .expect("block fallback reconciler should build");

    let mut resolver =
        KolmeRuntimeCommitForkFinalityResolver::new(notifications_consumer, block_reconciler);
    let receipt = resolver
        .resolve_commit_finality("kolme-commit:ab12cd34", 40, 45)
        .expect("resolver should use new-block height to drive fallback");

    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34:h42");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
    let observed_heights = block_calls
        .borrow()
        .iter()
        .map(|(_, _, height)| *height)
        .collect::<Vec<_>>();
    assert_eq!(observed_heights, vec![40, 41, 42]);
}

#[test]
fn regression_fork_finality_resolver_fails_closed_on_notification_txhash_mismatch() {
    // Regression: #1503
    let connector = MockConnector::new(vec![Ok(MockConnection::new(vec![MockStep::Message(
        "{\"NewBlock\":{\"block\":{\"txhash\":\"ff00\"},\"height\":42}}".to_owned(),
    )]))]);
    let notifications_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        1,
        connector,
    )
    .expect("notifications consumer should build");

    let (block_transport, block_calls) = RecordingBlockFallbackTransport::with_responses(vec![Ok(
        "{\"provider\":\"kolme-fork-local\",\"block_height\":42,\"tx_hashes\":\"ab12cd34\"}"
            .to_owned(),
    )]);
    let block_reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        2,
        block_transport,
    )
    .expect("block fallback reconciler should build");

    let mut resolver =
        KolmeRuntimeCommitForkFinalityResolver::new(notifications_consumer, block_reconciler);
    let error = resolver
        .resolve_commit_finality("kolme-commit:ab12cd34", 42, 42)
        .expect_err("txhash mismatch must fail closed");

    assert!(matches!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse { .. }
    ));
    assert!(
        block_calls.borrow().is_empty(),
        "fallback should not run after malformed notification mismatch"
    );
}

#[test]
fn regression_fork_finality_resolver_rejects_invalid_commit_id_shape() {
    // Regression: #1503
    let connector = MockConnector::new(vec![Ok(MockConnection::new(vec![MockStep::Closed]))]);
    let notifications_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        1,
        connector,
    )
    .expect("notifications consumer should build");

    let (block_transport, _block_calls) = RecordingBlockFallbackTransport::with_responses(vec![]);
    let block_reconciler = KolmeRuntimeCommitBlockFallbackReconciler::new(
        "http://127.0.0.1:3030",
        "/block/{height}",
        "kolme-fork-local",
        1,
        block_transport,
    )
    .expect("block fallback reconciler should build");

    let mut resolver =
        KolmeRuntimeCommitForkFinalityResolver::new(notifications_consumer, block_reconciler);
    assert_eq!(
        resolver.resolve_commit_finality("ab12cd34", 42, 42),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "commit_id must start with 'kolme-commit:'".to_owned(),
        })
    );
}
