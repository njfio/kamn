use kamn_core::{
    AdapterBackedKolmeRuntimeCommitClient, InMemoryKolmeRuntimeCommitClient,
    KolmeCommitReceiptFinality, KolmeRuntimeCommitClient, KolmeRuntimeCommitError,
    KolmeRuntimeCommitFinalityChecker, KolmeRuntimeCommitFinalityTransport,
    KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitProvider,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderTransport,
    KolmeRuntimeCommitRequest, KolmeRuntimeCommitSignedBroadcastEnvelope,
    KolmeRuntimeCommitTransportErrorKind, RuntimeCommitLifecycleState, RuntimeCommitPipeline,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Instant;

const FIXTURE: &str =
    include_str!("../../../fixtures/kolme_commit/runtime_commit_request_cases.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureCase {
    case_id: String,
    operation_id: String,
    state_root: String,
    actor_did: String,
    nonce: u64,
    payload_hash: String,
    expected_status: String,
    expected_reason: String,
}

type ProviderCalls = Rc<RefCell<Vec<(String, String)>>>;
type TransportCalls = Rc<RefCell<Vec<(String, String, String, String)>>>;
type FinalityTransportCalls = Rc<RefCell<Vec<(String, String, String)>>>;

#[derive(Debug, Clone)]
struct RecordingProvider {
    calls: ProviderCalls,
    result: Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>,
}

impl RecordingProvider {
    fn with_result(
        result: Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>,
    ) -> (Self, ProviderCalls) {
        let calls = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                result,
            },
            calls,
        )
    }
}

impl KolmeRuntimeCommitProvider for RecordingProvider {
    fn submit_runtime_commit(
        &mut self,
        wire_payload: &str,
        idempotency_key: &str,
    ) -> Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError> {
        self.calls
            .borrow_mut()
            .push((wire_payload.to_owned(), idempotency_key.to_owned()));
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct RecordingTransport {
    calls: TransportCalls,
    result: Result<String, KolmeRuntimeCommitProviderError>,
}

impl RecordingTransport {
    fn with_result(
        result: Result<String, KolmeRuntimeCommitProviderError>,
    ) -> (Self, TransportCalls) {
        let calls = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                calls: calls.clone(),
                result,
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
        self.result.clone()
    }
}

#[derive(Debug, Clone)]
struct RecordingFinalityTransport {
    calls: FinalityTransportCalls,
    responses: Rc<RefCell<VecDeque<Result<String, KolmeRuntimeCommitProviderError>>>>,
}

impl RecordingFinalityTransport {
    fn with_responses(
        responses: Vec<Result<String, KolmeRuntimeCommitProviderError>>,
    ) -> (Self, FinalityTransportCalls) {
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

impl KolmeRuntimeCommitFinalityTransport for RecordingFinalityTransport {
    fn fetch_runtime_commit_finality(
        &mut self,
        base_url: &str,
        status_path: &str,
        commit_id: &str,
    ) -> Result<String, KolmeRuntimeCommitProviderError> {
        self.calls.borrow_mut().push((
            base_url.to_owned(),
            status_path.to_owned(),
            commit_id.to_owned(),
        ));
        self.responses.borrow_mut().pop_front().unwrap_or_else(|| {
            Err(KolmeRuntimeCommitProviderError::Unavailable {
                reason: "no queued finality response".to_owned(),
            })
        })
    }
}

fn parse_fixture_cases() -> Vec<FixtureCase> {
    let mut cases = Vec::new();
    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line
            .split('|')
            .map(str::trim)
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert_eq!(columns.len(), 8, "fixture line must contain eight columns");
        cases.push(FixtureCase {
            case_id: columns[0].clone(),
            operation_id: columns[1].clone(),
            state_root: columns[2].clone(),
            actor_did: columns[3].clone(),
            nonce: columns[4]
                .parse::<u64>()
                .expect("nonce must be a valid unsigned integer"),
            payload_hash: columns[5].clone(),
            expected_status: columns[6].clone(),
            expected_reason: columns[7].clone(),
        });
    }
    cases
}

#[test]
fn unit_commit_request_wire_payload_is_deterministic() {
    let first = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-100",
        "state:abc123",
        "kamn:did:agent:runtime-node-1",
        7,
        "payload:stable",
    )
    .expect("first request should build");

    let second = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-100",
        "state:abc123",
        "kamn:did:agent:runtime-node-1",
        7,
        "payload:stable",
    )
    .expect("second request should build");

    assert_eq!(first.idempotency_key(), second.idempotency_key());
    assert_eq!(first.to_wire_payload(), second.to_wire_payload());
}

#[test]
fn functional_in_memory_commit_client_returns_submitted_then_duplicate() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-101",
        "state:abc123",
        "kamn:did:agent:runtime-node-2",
        2,
        "payload:functional",
    )
    .expect("request should build");

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let first = client
        .submit_commit(&request)
        .expect("first submit should succeed");
    let second = client
        .submit_commit(&request)
        .expect("duplicate submit should succeed");

    assert!(matches!(first, KolmeRuntimeCommitOutcome::Submitted(_)));
    assert!(matches!(second, KolmeRuntimeCommitOutcome::Duplicate(_)));
}

#[test]
fn integration_fixture_validator_classifies_commit_request_schema_cases() {
    let cases = parse_fixture_cases();
    assert_eq!(cases.len(), 5);

    for case in cases {
        let result = KolmeRuntimeCommitRequest::deterministic(
            case.operation_id.as_str(),
            case.state_root.as_str(),
            case.actor_did.as_str(),
            case.nonce,
            case.payload_hash.as_str(),
        );

        match case.expected_status.as_str() {
            "pass" => {
                assert!(
                    result.is_ok(),
                    "expected case '{}' to pass, got {result:?}",
                    case.case_id
                );
            }
            "fail" => match result {
                Err(KolmeRuntimeCommitError::InvalidRequest { reason, .. }) => {
                    assert_eq!(
                        reason,
                        case.expected_reason.as_str(),
                        "fixture reason mismatch for case {}",
                        case.case_id
                    );
                }
                other => panic!(
                    "expected invalid request error for case '{}' but got {other:?}",
                    case.case_id
                ),
            },
            other => panic!("unexpected fixture expected_status value: {other}"),
        }
    }
}

#[test]
fn regression_submit_commit_fails_closed_for_mutated_invalid_request() {
    // Regression: #825
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-102",
        "state:abc123",
        "kamn:did:agent:runtime-node-3",
        3,
        "payload:valid",
    )
    .expect("request should build");
    request.payload_hash.clear();

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "payload_hash",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1892_submit_commit_fails_closed_for_empty_operation_id() {
    // Regression: #1892
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1892-operation",
        "state:op",
        "kamn:did:agent:runtime-node-1892",
        13,
        "payload:op",
    )
    .expect("request should build");
    request.operation_id.clear();

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "operation_id",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1892_submit_commit_fails_closed_for_empty_state_root() {
    // Regression: #1892
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1892-state-root",
        "state:op",
        "kamn:did:agent:runtime-node-1892",
        14,
        "payload:state",
    )
    .expect("request should build");
    request.state_root.clear();

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "state_root",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1894_submit_commit_fails_closed_for_multiline_operation_id() {
    // Regression: #1894
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1894-multiline",
        "state:op",
        "kamn:did:agent:runtime-node-1894",
        15,
        "payload:op",
    )
    .expect("request should build");
    request.operation_id.push_str("\nwrapped");

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "wire_payload",
            reason: "fields must be single-line",
        })
    );
}

#[test]
fn regression_issue_1896_signed_envelope_constructor_rejects_empty_fields() {
    // Regression: #1896
    assert_eq!(
        KolmeRuntimeCommitSignedBroadcastEnvelope::new(" ", "operation_id=op\n", "sig-1", 1),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "signer_key_id",
            reason: "must not be empty",
        })
    );
    assert_eq!(
        KolmeRuntimeCommitSignedBroadcastEnvelope::new("kamn:key:signer:1", " ", "sig-1", 1),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "signed_message",
            reason: "must not be empty",
        })
    );
    assert_eq!(
        KolmeRuntimeCommitSignedBroadcastEnvelope::new(
            "kamn:key:signer:1",
            "operation_id=op\n",
            " ",
            1
        ),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "signature",
            reason: "must not be empty",
        })
    );
}

#[test]
fn regression_issue_1900_submit_commit_fails_closed_for_zero_nonce() {
    // Regression: #1900
    let mut request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-1900-nonce",
        "state:nonce",
        "kamn:did:agent:runtime-node-1900",
        1,
        "payload:nonce",
    )
    .expect("request should build");
    request.nonce = 0;

    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    assert_eq!(
        client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "nonce",
            reason: "must be positive",
        })
    );
}

#[test]
fn performance_runtime_commit_contract_lane_stays_within_budget() {
    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let started = Instant::now();

    for nonce in 1..=256 {
        let request = KolmeRuntimeCommitRequest::deterministic(
            format!("op-sync-perf-{nonce}").as_str(),
            "state:perf",
            "kamn:did:agent:runtime-node-perf",
            nonce,
            format!("payload:perf:{nonce}").as_str(),
        )
        .expect("request should build");

        let outcome = client
            .submit_commit(&request)
            .expect("submit should succeed");
        assert!(matches!(outcome, KolmeRuntimeCommitOutcome::Submitted(_)));
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 300,
        "runtime commit contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
fn unit_adapter_normalizes_wire_payload_and_idempotency_key_before_submit() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-adapter-100",
        "state:adapter",
        "kamn:did:agent:runtime-node-adapter-1",
        9,
        "payload:adapter",
    )
    .expect("request should build");
    let expected_payload = request.to_wire_payload();
    let expected_key = request.idempotency_key().to_owned();

    let (provider, calls) = RecordingProvider::with_result(Ok(
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-local".to_owned(),
            commit_id: "kolme-commit:adapter-100".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }),
    ));
    let mut client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", provider).expect("client");

    let outcome = client
        .submit_commit(&request)
        .expect("adapter submit should succeed");
    assert!(matches!(outcome, KolmeRuntimeCommitOutcome::Submitted(_)));

    let calls = calls.borrow();
    assert_eq!(
        calls.len(),
        1,
        "adapter must submit exactly one provider call"
    );
    assert_eq!(calls[0].0, expected_payload);
    assert_eq!(calls[0].1, expected_key);
}

#[test]
fn functional_adapter_maps_transport_provider_and_finality_failures_to_typed_errors() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-adapter-101",
        "state:adapter",
        "kamn:did:agent:runtime-node-adapter-2",
        4,
        "payload:adapter",
    )
    .expect("request should build");

    let (timeout_provider, _calls) =
        RecordingProvider::with_result(Err(KolmeRuntimeCommitProviderError::Timeout));
    let mut timeout_client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", timeout_provider)
            .expect("client");
    assert_eq!(
        timeout_client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::ProviderTransport {
            kind: KolmeRuntimeCommitTransportErrorKind::Timeout,
            detail: "provider request timed out".to_owned(),
        })
    );

    let (mismatch_provider, _calls) = RecordingProvider::with_result(Ok(
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-remote".to_owned(),
            commit_id: "kolme-commit:adapter-101".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }),
    ));
    let mut mismatch_client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", mismatch_provider)
            .expect("client");
    assert_eq!(
        mismatch_client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::ProviderMismatch {
            expected: "kolme-local".to_owned(),
            observed: "kolme-remote".to_owned(),
        })
    );

    let (pending_provider, _calls) = RecordingProvider::with_result(Ok(
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-local".to_owned(),
            commit_id: "kolme-commit:adapter-102".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        }),
    ));
    let mut pending_client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", pending_provider)
            .expect("client");
    assert_eq!(
        pending_client.submit_commit(&request),
        Err(KolmeRuntimeCommitError::NonFinalReceipt {
            commit_id: "kolme-commit:adapter-102".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        })
    );
}

#[test]
fn integration_runtime_pipeline_accepts_adapter_backed_final_receipts() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-adapter-103",
        "state:adapter",
        "kamn:did:agent:runtime-node-adapter-3",
        5,
        "payload:adapter",
    )
    .expect("request should build");

    let (provider, _calls) = RecordingProvider::with_result(Ok(
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-local".to_owned(),
            commit_id: "kolme-commit:adapter-103".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }),
    ));
    let mut client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", provider).expect("client");
    let mut pipeline = RuntimeCommitPipeline::new();

    let record = pipeline
        .submit_with_client(&mut client, request)
        .expect("pipeline submit should succeed");
    assert_eq!(record.state, RuntimeCommitLifecycleState::Finalized);
    assert!(!record.needs_requeue);
}

#[test]
fn regression_adapter_path_keeps_receipt_provider_mismatch_fail_closed() {
    // Regression: #979
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-sync-adapter-104",
        "state:adapter",
        "kamn:did:agent:runtime-node-adapter-4",
        6,
        "payload:adapter",
    )
    .expect("request should build");

    let (provider, _calls) = RecordingProvider::with_result(Ok(
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-local".to_owned(),
            commit_id: "kolme-commit:adapter-104".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }),
    ));
    let mut client =
        AdapterBackedKolmeRuntimeCommitClient::new("kolme-local", provider).expect("client");
    let mut pipeline = RuntimeCommitPipeline::new();

    let record = pipeline
        .submit_with_client(&mut client, request)
        .expect("pipeline submit should succeed");
    assert_eq!(record.receipt_provider.as_deref(), Some("kolme-local"));
    assert_eq!(
        pipeline.apply_receipt_finality(
            "op-sync-adapter-104",
            KolmeCommitReceiptFinality::Final,
            "kolme-remote",
            "kolme-commit:adapter-104",
        ),
        Err(KolmeRuntimeCommitError::ReceiptFieldMismatch {
            field: "receipt_provider",
            expected: "kolme-local".to_owned(),
            observed: "kolme-remote".to_owned(),
        })
    );
}

#[test]
fn unit_live_provider_rejects_empty_endpoint_or_submit_path() {
    let (transport, _calls) = RecordingTransport::with_result(Ok(String::new()));
    assert!(
        matches!(
            KolmeRuntimeCommitLiveProvider::new("", "/broadcast/runtime-commit", transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            })
        ),
        "provider base URL should fail validation when empty"
    );

    let (transport, _calls) = RecordingTransport::with_result(Ok(String::new()));
    assert!(
        matches!(
            KolmeRuntimeCommitLiveProvider::new("http://127.0.0.1:3030", "", transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_submit_path",
                reason: "must not be empty",
            })
        ),
        "provider submit path should fail validation when empty"
    );
}

#[test]
fn unit_adapter_backed_client_rejects_empty_expected_provider() {
    let (provider, _calls) =
        RecordingProvider::with_result(Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "unused".to_owned(),
        }));
    assert!(
        matches!(
            AdapterBackedKolmeRuntimeCommitClient::new(" ", provider),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "expected_provider",
                reason: "must not be empty",
            })
        ),
        "expected provider should fail validation when empty"
    );
}

#[test]
fn unit_in_memory_client_rejects_empty_provider() {
    assert!(
        matches!(
            InMemoryKolmeRuntimeCommitClient::new(""),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider",
                reason: "must not be empty",
            })
        ),
        "in-memory provider should fail validation when empty"
    );
}

#[test]
fn functional_live_provider_maps_submitted_json_response_to_provider_outcome() {
    let response = r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:runtime:55","finality":"final"}"#.to_owned();
    let (transport, calls) = RecordingTransport::with_result(Ok(response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-001",
        "state:live",
        "kamn:did:agent:live-provider-1",
        55,
        "payload:live-provider",
    )
    .expect("request should build");

    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should return a parsed outcome");
    assert_eq!(
        outcome,
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:runtime:55".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        })
    );

    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "provider transport should be called once");
    assert_eq!(calls[0].0, "http://127.0.0.1:3030");
    assert_eq!(calls[0].1, "/broadcast/runtime-commit");
    assert_eq!(calls[0].2, request.to_wire_payload());
    assert_eq!(calls[0].3, request.idempotency_key());
}

#[test]
fn regression_issue_1920_live_provider_trims_endpoint_inputs() {
    // Regression: #1920
    let response = r#"{"status":"submitted","provider":"kolme-fork-local","commit_id":"kolme-commit:runtime:1920","finality":"pending"}"#.to_owned();
    let (transport, calls) = RecordingTransport::with_result(Ok(response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "  http://127.0.0.1:3030  ",
        "  /broadcast/runtime-commit  ",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-1920",
        "state:live",
        "kamn:did:agent:live-provider-1920",
        61,
        "payload:live-provider-1920",
    )
    .expect("request should build");

    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should return a parsed outcome");
    assert_eq!(
        outcome,
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:runtime:1920".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        })
    );

    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "provider transport should be called once");
    assert_eq!(calls[0].0, "http://127.0.0.1:3030");
    assert_eq!(calls[0].1, "/broadcast/runtime-commit");
}

#[test]
fn unit_kolme_fork_live_provider_maps_txhash_only_response_using_provider_hint() {
    let response = r#"{"txhash":"ab12cd34"}"#.to_owned();
    let (transport, calls) = RecordingTransport::with_result(Ok(response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-1502-a",
        "state:live",
        "kamn:did:agent:live-provider-1502-a",
        59,
        "payload:live-provider",
    )
    .expect("request should build");

    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should map txhash-only response");
    assert_eq!(
        outcome,
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        })
    );

    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "provider transport should be called once");
    assert_eq!(calls[0].1, "/broadcast");
}

#[test]
fn regression_live_provider_fails_closed_for_malformed_response_shape() {
    // Regression: #1411
    let malformed_response =
        r#"{"status":"submitted","provider":"kolme-fork-local","finality":"final"}"#.to_owned();
    let (transport, _calls) = RecordingTransport::with_result(Ok(malformed_response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-002",
        "state:live",
        "kamn:did:agent:live-provider-2",
        56,
        "payload:live-provider",
    )
    .expect("request should build");

    assert!(
        matches!(
            provider.submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key()),
            Err(KolmeRuntimeCommitProviderError::MalformedResponse { .. })
        ),
        "provider must fail closed for malformed backend responses"
    );
}

#[test]
fn regression_live_provider_rejects_statusless_response_without_txhash() {
    // Regression: #1502
    let malformed_response =
        r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:runtime:missing-status"}"#
            .to_owned();
    let (transport, _calls) = RecordingTransport::with_result(Ok(malformed_response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-1502-b",
        "state:live",
        "kamn:did:agent:live-provider-1502-b",
        60,
        "payload:live-provider",
    )
    .expect("request should build");

    assert!(
        matches!(
            provider.submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key()),
            Err(KolmeRuntimeCommitProviderError::MalformedResponse { .. })
        ),
        "provider must fail closed when neither status nor txhash is present"
    );
}

#[test]
fn functional_live_provider_maps_tx_hash_and_block_height_to_deterministic_commit_id() {
    let response = r#"{"status":"submitted","provider":"kolme-fork-local","tx_hash":"ab12cd34","block_height":"42","finality":"confirmed"}"#.to_owned();
    let (transport, _calls) = RecordingTransport::with_result(Ok(response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-003",
        "state:live",
        "kamn:did:agent:live-provider-3",
        57,
        "payload:live-provider",
    )
    .expect("request should build");

    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should map tx_hash response");
    assert_eq!(
        outcome,
        KolmeRuntimeCommitProviderOutcome::Submitted(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34:h42".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        })
    );
}

#[test]
fn regression_live_provider_normalizes_backend_finality_aliases() {
    // Regression: #1412
    let response = r#"{"status":"duplicate","provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"accepted"}"#.to_owned();
    let (transport, _calls) = RecordingTransport::with_result(Ok(response));
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:3030",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-live-provider-004",
        "state:live",
        "kamn:did:agent:live-provider-4",
        58,
        "payload:live-provider",
    )
    .expect("request should build");

    let outcome = provider
        .submit_runtime_commit(&request.to_wire_payload(), request.idempotency_key())
        .expect("provider should map finality alias");
    assert_eq!(
        outcome,
        KolmeRuntimeCommitProviderOutcome::Duplicate(KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34:h42".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        })
    );
}

#[test]
fn unit_finality_checker_rejects_empty_endpoint_or_status_path() {
    let (transport, _calls) = RecordingFinalityTransport::with_responses(Vec::new());
    assert!(
        matches!(
            KolmeRuntimeCommitFinalityChecker::new("", "/commit/finality", transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_base_url",
                reason: "must not be empty",
            })
        ),
        "finality checker base URL should fail validation when empty"
    );

    let (transport, _calls) = RecordingFinalityTransport::with_responses(Vec::new());
    assert!(
        matches!(
            KolmeRuntimeCommitFinalityChecker::new("http://127.0.0.1:3030", "", transport),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "provider_status_path",
                reason: "must not be empty",
            })
        ),
        "finality checker status path should fail validation when empty"
    );
}

#[test]
fn functional_finality_checker_maps_confirmed_alias_to_final_receipt() {
    let response = r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"confirmed"}"#.to_owned();
    let (transport, calls) = RecordingFinalityTransport::with_responses(vec![Ok(response)]);
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        "http://127.0.0.1:3030",
        "/commit/finality",
        transport,
    )
    .expect("checker should build");

    let receipt = checker
        .check_commit_finality("kolme-commit:ab12cd34:h42")
        .expect("checker should parse finality response");
    assert_eq!(
        receipt,
        KolmeRuntimeCommitProviderReceipt {
            provider: "kolme-fork-local".to_owned(),
            commit_id: "kolme-commit:ab12cd34:h42".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }
    );

    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "finality transport should be called once");
    assert_eq!(calls[0].0, "http://127.0.0.1:3030");
    assert_eq!(calls[0].1, "/commit/finality");
    assert_eq!(calls[0].2, "kolme-commit:ab12cd34:h42");
}

#[test]
fn regression_issue_1918_finality_checker_trims_endpoint_inputs() {
    // Regression: #1918
    let response = r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"pending"}"#.to_owned();
    let (transport, calls) = RecordingFinalityTransport::with_responses(vec![Ok(response)]);
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        "  http://127.0.0.1:3030  ",
        "  /commit/finality  ",
        transport,
    )
    .expect("checker should build");

    let receipt = checker
        .check_commit_finality("kolme-commit:ab12cd34:h42")
        .expect("checker should parse finality response");
    assert_eq!(receipt.provider, "kolme-fork-local");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);

    let calls = calls.borrow();
    assert_eq!(calls.len(), 1, "finality transport should be called once");
    assert_eq!(calls[0].0, "http://127.0.0.1:3030");
    assert_eq!(calls[0].1, "/commit/finality");
}

#[test]
fn functional_finality_checker_polls_pending_then_final() {
    let pending =
        r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"pending"}"#.to_owned();
    let confirmed =
        r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"confirmed"}"#.to_owned();
    let (transport, _calls) =
        RecordingFinalityTransport::with_responses(vec![Ok(pending), Ok(confirmed)]);
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        "http://127.0.0.1:3030",
        "/commit/finality",
        transport,
    )
    .expect("checker should build");

    let receipt = checker
        .poll_finality("kolme-commit:ab12cd34:h42", 2)
        .expect("checker should return first non-pending finality");
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_finality_checker_fails_closed_for_commit_id_mismatch() {
    // Regression: #1413
    let mismatch =
        r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:other:h42","finality":"final"}"#.to_owned();
    let (transport, _calls) = RecordingFinalityTransport::with_responses(vec![Ok(mismatch)]);
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        "http://127.0.0.1:3030",
        "/commit/finality",
        transport,
    )
    .expect("checker should build");

    assert!(
        matches!(
            checker.check_commit_finality("kolme-commit:ab12cd34:h42"),
            Err(KolmeRuntimeCommitProviderError::MalformedResponse { .. })
        ),
        "checker must fail closed when response commit id mismatches requested commit id"
    );
}

#[test]
fn regression_finality_checker_times_out_when_pending_budget_exhausted() {
    // Regression: #1413
    let pending =
        r#"{"provider":"kolme-fork-local","commit_id":"kolme-commit:ab12cd34:h42","finality":"pending"}"#.to_owned();
    let (transport, _calls) =
        RecordingFinalityTransport::with_responses(vec![Ok(pending.clone()), Ok(pending)]);
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        "http://127.0.0.1:3030",
        "/commit/finality",
        transport,
    )
    .expect("checker should build");

    assert_eq!(
        checker.poll_finality("kolme-commit:ab12cd34:h42", 2),
        Err(KolmeRuntimeCommitProviderError::Timeout)
    );
}
