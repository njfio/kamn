use kamn_kolme::{
    build_kolme_fork_broadcast_live_provider_config, build_runtime_commit_live_provider_config,
    submit_runtime_commit_live_provider_request, KolmeCommitReceiptFinality,
    KolmeRuntimeCommitLiveProviderConfigError, KolmeRuntimeCommitProviderError,
    KolmeRuntimeCommitProviderTransport, KolmeRuntimeProviderOutcome,
};
use std::cell::RefCell;
use std::rc::Rc;

type TransportCalls = Rc<RefCell<Vec<(String, String, String, String)>>>;

#[derive(Debug, Clone)]
struct RecordingTransport {
    calls: TransportCalls,
    response: Result<String, KolmeRuntimeCommitProviderError>,
}

impl RecordingTransport {
    fn with_response(
        response: Result<String, KolmeRuntimeCommitProviderError>,
    ) -> (Self, TransportCalls) {
        let calls = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                response,
            },
            calls,
        )
    }
}

impl KolmeRuntimeCommitProviderTransport for RecordingTransport {
    fn submit_runtime_commit(
        &mut self,
        base_url: &str,
        submit_path: &str,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        self.calls.borrow_mut().push((
            base_url.to_owned(),
            submit_path.to_owned(),
            wire_payload.to_owned(),
            idempotency_key.to_owned(),
        ));
        self.response.clone()
    }
}

#[test]
fn unit_live_provider_pipeline_rejects_empty_config_inputs() {
    assert_eq!(
        build_runtime_commit_live_provider_config("", "/runtime-commit")
            .expect_err("base URL must fail"),
        KolmeRuntimeCommitLiveProviderConfigError::InvalidRequest {
            field: "provider_base_url",
            reason: "must not be empty",
        }
    );
    assert_eq!(
        build_runtime_commit_live_provider_config("http://127.0.0.1:3000", "   ")
            .expect_err("submit path must fail"),
        KolmeRuntimeCommitLiveProviderConfigError::InvalidRequest {
            field: "provider_submit_path",
            reason: "must not be empty",
        }
    );
    assert_eq!(
        build_kolme_fork_broadcast_live_provider_config("http://127.0.0.1:3000", "  ")
            .expect_err("provider hint must fail"),
        KolmeRuntimeCommitLiveProviderConfigError::InvalidRequest {
            field: "provider_hint",
            reason: "must not be empty",
        }
    );
}

#[test]
fn functional_live_provider_pipeline_normalizes_config_and_maps_submitted_outcome() {
    let config = build_runtime_commit_live_provider_config(
        "  http://127.0.0.1:3000  ",
        "  /runtime-commit/submit  ",
    )
    .expect("config should normalize");
    let (mut transport, calls) = RecordingTransport::with_response(Ok(
        "status=submitted\nprovider=kolme-fork-local\ncommit_id=kolme-commit:ab12cd34\nfinality=final\n"
            .to_owned(),
    ));
    let outcome = submit_runtime_commit_live_provider_request(
        &mut transport,
        &config,
        "operation_id=op-77\n",
        "idempotency-key-77",
    )
    .expect("provider response should parse");

    assert_eq!(calls.borrow().len(), 1);
    assert_eq!(
        calls.borrow()[0],
        (
            "http://127.0.0.1:3000".to_owned(),
            "/runtime-commit/submit".to_owned(),
            "operation_id=op-77\n".to_owned(),
            "idempotency-key-77".to_owned(),
        )
    );
    assert_eq!(
        outcome,
        KolmeRuntimeProviderOutcome::Submitted {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }
    );
}

#[test]
fn functional_live_provider_pipeline_fork_profile_uses_provider_hint_for_txhash_shape() {
    let config = build_kolme_fork_broadcast_live_provider_config(
        "http://127.0.0.1:3000",
        "  kolme-fork-local  ",
    )
    .expect("fork config should normalize");
    let (mut transport, _) =
        RecordingTransport::with_response(Ok("{\"txhash\":\"ab12cd34\"}".to_owned()));
    let outcome = submit_runtime_commit_live_provider_request(
        &mut transport,
        &config,
        "message={}\n",
        "idempotency-key-fork",
    )
    .expect("txhash-only response should map via provider hint");

    assert_eq!(
        outcome,
        KolmeRuntimeProviderOutcome::Submitted {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        }
    );
}

#[test]
fn regression_issue_2278_live_provider_pipeline_fails_closed_for_malformed_response() {
    // Regression: #2278
    let config = build_runtime_commit_live_provider_config(
        "http://127.0.0.1:3000",
        "/runtime-commit/submit",
    )
    .expect("config should build");
    let (mut transport, _) = RecordingTransport::with_response(Ok("{}".to_owned()));
    let error = submit_runtime_commit_live_provider_request(
        &mut transport,
        &config,
        "operation_id=op-1\n",
        "idempotency-key-1",
    )
    .expect_err("malformed response must fail closed");

    assert!(matches!(
        error,
        KolmeRuntimeCommitProviderError::MalformedResponse { .. }
    ));
}
