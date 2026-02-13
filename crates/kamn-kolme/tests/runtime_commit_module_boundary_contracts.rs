use kamn_kolme::{
    resolve_finality, EchoTransport, FinalityState, KolmeApiBroadcastRequest,
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitFinalityTransport,
    KolmeRuntimeCommitNotificationsConnection, KolmeRuntimeCommitNotificationsConnector,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderTransport, PassthroughCodec,
    RuntimeCommitPipeline,
};

struct StubRuntimeTransport;

impl KolmeRuntimeCommitProviderTransport for StubRuntimeTransport {
    fn submit_runtime_commit(
        &mut self,
        _base_url: &str,
        _submit_path: &str,
        _wire_payload: &str,
        _idempotency_key: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        Ok("{\"txhash\":\"ab12cd34\"}".to_owned())
    }
}

impl KolmeRuntimeCommitFinalityTransport for StubRuntimeTransport {
    fn fetch_runtime_commit_finality(
        &mut self,
        _base_url: &str,
        _status_path: &str,
        _commit_id: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        Ok("{\"finality\":\"final\"}".to_owned())
    }
}

impl KolmeRuntimeCommitBlockFallbackTransport for StubRuntimeTransport {
    fn fetch_block_by_height(
        &mut self,
        _base_url: &str,
        _block_path_template: &str,
        _height: u64,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        Ok("{\"height\":1}".to_owned())
    }
}

struct StubNotificationsConnection;

impl KolmeRuntimeCommitNotificationsConnection for StubNotificationsConnection {
    fn read_text_message(&mut self) -> Result<Option<String>, KolmeRuntimeCommitProviderError> {
        Ok(None)
    }
}

struct StubNotificationsConnector;

impl KolmeRuntimeCommitNotificationsConnector for StubNotificationsConnector {
    type Connection = StubNotificationsConnection;

    fn connect(
        &mut self,
        _notifications_url: &str,
    ) -> Result<Self::Connection, KolmeRuntimeCommitProviderError> {
        Ok(StubNotificationsConnection)
    }
}

#[test]
fn runtime_commit_module_boundary_exports_transport_and_notifications_traits() {
    let mut transport = StubRuntimeTransport;
    let submit = transport
        .submit_runtime_commit(
            "http://127.0.0.1:3000",
            "/broadcast",
            "{\"message\":\"{}\"}",
            "idem-1",
        )
        .expect("provider transport trait should be callable");
    assert!(submit.contains("\"txhash\""));

    let finality = transport
        .fetch_runtime_commit_finality(
            "http://127.0.0.1:3000",
            "/runtime-commit/status",
            "kolme-commit:ab12cd34",
        )
        .expect("finality transport trait should be callable");
    assert!(finality.contains("\"finality\""));

    let block = transport
        .fetch_block_by_height("http://127.0.0.1:3000", "/block/{height}", 1)
        .expect("block fallback transport trait should be callable");
    assert!(block.contains("\"height\""));

    let mut connector = StubNotificationsConnector;
    let mut connection = connector
        .connect("ws://127.0.0.1:3000/notifications")
        .expect("notifications connector trait should be callable");
    assert_eq!(
        connection
            .read_text_message()
            .expect("notifications connection trait should be callable"),
        None
    );
}

#[test]
fn runtime_commit_module_boundary_exports_codec_finality_and_pipeline_modules() {
    let finality = resolve_finality(2, 1, false);
    assert_eq!(finality.state(), FinalityState::Confirmed);

    let broadcast = KolmeApiBroadcastRequest::new("{\"message\":\"hello\"}", "ab12", 0)
        .expect("api codec constructor should be exported");
    assert!(broadcast.to_json_payload().contains("\"signature\""));

    let pipeline = RuntimeCommitPipeline::new(PassthroughCodec, EchoTransport);
    let output = pipeline
        .submit("http://127.0.0.1:3000/tx", b"runtime-commit", 1, false)
        .expect("pipeline module should be exported");
    assert_eq!(output, b"runtime-commit");
}
