pub(super) use kamn_core::{
    AdapterBackedKolmeRuntimeCommitClient, InMemoryKolmeRuntimeCommitClient,
    KolmeCommitReceiptFinality, KolmeRuntimeCommitClient, KolmeRuntimeCommitError,
    KolmeRuntimeCommitFinalityChecker, KolmeRuntimeCommitFinalityTransport,
    KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitProvider,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome,
    KolmeRuntimeCommitProviderReceipt, KolmeRuntimeCommitProviderTransport,
    KolmeRuntimeCommitRequest, KolmeRuntimeCommitSignedBroadcastEnvelope,
    KolmeRuntimeCommitTransportErrorKind, RuntimeCommitLifecycleState, RuntimeCommitPipeline,
};
pub(super) use kamn_kolme::KolmeRuntimeProviderOutcome as KamnKolmeRuntimeProviderOutcome;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
pub(super) use std::time::Instant;

const FIXTURE: &str =
    include_str!("../../../../fixtures/kolme_commit/runtime_commit_request_cases.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FixtureCase {
    pub(super) case_id: String,
    pub(super) operation_id: String,
    pub(super) state_root: String,
    pub(super) actor_did: String,
    pub(super) nonce: u64,
    pub(super) payload_hash: String,
    pub(super) expected_status: String,
    pub(super) expected_reason: String,
}

pub(super) type ProviderCalls = Rc<RefCell<Vec<(String, String)>>>;
pub(super) type TransportCalls = Rc<RefCell<Vec<(String, String, String, String)>>>;
pub(super) type FinalityTransportCalls = Rc<RefCell<Vec<(String, String, String)>>>;

#[derive(Debug, Clone)]
pub(super) struct RecordingProvider {
    calls: ProviderCalls,
    result: Result<KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderError>,
}

impl RecordingProvider {
    pub(super) fn with_result(
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
pub(super) struct RecordingTransport {
    calls: TransportCalls,
    result: Result<String, KolmeRuntimeCommitProviderError>,
}

impl RecordingTransport {
    pub(super) fn with_result(
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
pub(super) struct RecordingFinalityTransport {
    calls: FinalityTransportCalls,
    responses: Rc<RefCell<VecDeque<Result<String, KolmeRuntimeCommitProviderError>>>>,
}

impl RecordingFinalityTransport {
    pub(super) fn with_responses(
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

pub(super) fn parse_fixture_cases() -> Vec<FixtureCase> {
    FIXTURE
        .lines()
        .filter_map(parse_fixture_case_line)
        .collect()
}

fn parse_fixture_case_line(raw_line: &str) -> Option<FixtureCase> {
    let line = raw_line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    Some(build_fixture_case(fixture_columns(line)))
}

fn fixture_columns(line: &str) -> Vec<String> {
    line.split('|')
        .map(str::trim)
        .map(str::to_owned)
        .collect::<Vec<_>>()
}

fn build_fixture_case(columns: Vec<String>) -> FixtureCase {
    assert_eq!(columns.len(), 8, "fixture line must contain eight columns");
    FixtureCase {
        case_id: columns[0].clone(),
        operation_id: columns[1].clone(),
        state_root: columns[2].clone(),
        actor_did: columns[3].clone(),
        nonce: parse_fixture_nonce(&columns[4]),
        payload_hash: columns[5].clone(),
        expected_status: columns[6].clone(),
        expected_reason: columns[7].clone(),
    }
}

fn parse_fixture_nonce(nonce: &str) -> u64 {
    nonce
        .parse::<u64>()
        .expect("nonce must be a valid unsigned integer")
}
