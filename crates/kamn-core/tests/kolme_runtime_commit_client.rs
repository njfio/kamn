use kamn_core::{
    InMemoryKolmeRuntimeCommitClient, KolmeRuntimeCommitClient, KolmeRuntimeCommitError,
    KolmeRuntimeCommitOutcome, KolmeRuntimeCommitRequest,
};
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
