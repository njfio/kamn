use kamn_core::{
    KolmeCommitReceiptFinality, KolmeRuntimeCommitNotificationEvent,
    KolmeRuntimeCommitNotificationsConnection, KolmeRuntimeCommitNotificationsConnector,
    KolmeRuntimeCommitNotificationsConsumer, KolmeRuntimeCommitProviderError,
    KolmeRuntimeCommitWebsocketConnector,
};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
enum MockStep {
    Message(String),
    Closed,
}

#[derive(Clone)]
struct MockConnection {
    steps: Arc<Mutex<VecDeque<MockStep>>>,
}

impl MockConnection {
    fn new(steps: Vec<MockStep>) -> Self {
        Self {
            steps: Arc::new(Mutex::new(VecDeque::from(steps))),
        }
    }
}

impl KolmeRuntimeCommitNotificationsConnection for MockConnection {
    fn read_text_message(&mut self) -> Result<Option<String>, KolmeRuntimeCommitProviderError> {
        let mut guard = self
            .steps
            .lock()
            .expect("mock connection lock should not be poisoned");
        let Some(step) = guard.pop_front() else {
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

fn spawn_mock_websocket_notification_server(notification_payload: &str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("mock websocket listener should bind");
    let port = listener
        .local_addr()
        .expect("mock websocket listener should expose local address")
        .port();
    let payload = notification_payload.to_owned();
    thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .expect("mock websocket listener should accept one client");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("mock websocket read timeout should be configured");

        let mut request = Vec::new();
        loop {
            let mut chunk = [0_u8; 256];
            let read = stream
                .read(&mut chunk)
                .expect("mock websocket should read handshake request");
            if read == 0 {
                panic!("mock websocket client disconnected before handshake completed");
            }
            request.extend_from_slice(&chunk[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        let request_text = String::from_utf8(request).expect("request bytes should be utf-8");
        assert!(
            request_text.starts_with("GET /notifications HTTP/1.1\r\n"),
            "expected websocket handshake request path /notifications, got: {request_text}"
        );
        assert!(
            request_text.contains("Upgrade: websocket"),
            "expected websocket upgrade header in request: {request_text}"
        );

        let handshake = concat!(
            "HTTP/1.1 101 Switching Protocols\r\n",
            "Upgrade: websocket\r\n",
            "Connection: Upgrade\r\n",
            "Sec-WebSocket-Accept: ignored\r\n",
            "\r\n"
        );
        stream
            .write_all(handshake.as_bytes())
            .expect("mock websocket handshake should be written");

        let payload_bytes = payload.as_bytes();
        assert!(
            payload_bytes.len() < 126,
            "test notification payload must fit one short websocket frame"
        );
        let mut frame = vec![0x81, payload_bytes.len() as u8];
        frame.extend_from_slice(payload_bytes);
        stream
            .write_all(frame.as_slice())
            .expect("mock websocket text frame should be written");
    });
    port
}

#[test]
fn functional_notifications_consumer_reconnects_after_closed_connection() {
    let first = Ok(MockConnection::new(vec![MockStep::Closed]));
    let second = Ok(MockConnection::new(vec![MockStep::Message(
        "{\"FailedTransaction\":{\"message\":{\"txhash\":\"ff22aa\"}}}".to_owned(),
    )]));
    let connector = MockConnector::new(vec![first, second]);
    let mut consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        2,
        connector,
    )
    .expect("notifications consumer should build");

    let receipt = consumer
        .next_commit_receipt()
        .expect("consumer should recover after one closed connection");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ff22aa");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Failed);
}

#[test]
fn functional_notifications_consumer_decodes_new_block_variant_to_final_receipt() {
    let connector = MockConnector::new(vec![Ok(MockConnection::new(vec![MockStep::Message(
        "{\"NewBlock\":{\"block\":{\"txhash\":\"ab12cd34\"},\"height\":42}}".to_owned(),
    )]))]);
    let mut consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        1,
        connector,
    )
    .expect("notifications consumer should build");

    let event = consumer
        .next_notification_event()
        .expect("notification event should decode");
    assert_eq!(
        event,
        KolmeRuntimeCommitNotificationEvent::NewBlock {
            txhash: "ab12cd34".to_owned(),
            block_height: Some(42),
        }
    );

    let receipt = event
        .to_provider_receipt("kolme-fork-local")
        .expect("new block event should map to final receipt");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34:h42");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn integration_notifications_websocket_connector_receives_kolme_notification() {
    let port = spawn_mock_websocket_notification_server(
        "{\"NewBlock\":{\"block\":{\"txhash\":\"00c0ffee\"},\"height\":88}}",
    );
    let connector =
        KolmeRuntimeCommitWebsocketConnector::new(2).expect("websocket connector should build");
    let mut consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        format!("http://127.0.0.1:{port}").as_str(),
        "/notifications",
        "kolme-fork-local",
        1,
        connector,
    )
    .expect("notifications consumer should build");

    let receipt = consumer
        .next_commit_receipt()
        .expect("websocket connector should yield one parsed receipt");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.commit_id, "kolme-commit:00c0ffee:h88");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_notifications_consumer_fails_closed_on_decode_and_retry_exhaustion() {
    // Regression: #1463
    let malformed_connector =
        MockConnector::new(vec![Ok(MockConnection::new(vec![MockStep::Message(
            "{\"UnknownVariant\":{\"txhash\":\"11\"}}".to_owned(),
        )]))]);
    let mut malformed_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        1,
        malformed_connector,
    )
    .expect("notifications consumer should build");

    assert_eq!(
        malformed_consumer.next_notification_event(),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "notification variant is unsupported".to_owned(),
        })
    );

    let retry_connector = MockConnector::new(vec![
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "dial failed: refused".to_owned(),
        }),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "dial failed: refused".to_owned(),
        }),
    ]);
    let mut retry_consumer = KolmeRuntimeCommitNotificationsConsumer::new(
        "http://127.0.0.1:3030",
        "/notifications",
        "kolme-fork-local",
        2,
        retry_connector,
    )
    .expect("notifications consumer should build");
    assert_eq!(
        retry_consumer.next_notification_event(),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "notification reconnect attempts exhausted after 2 retries".to_owned(),
        })
    );
}
