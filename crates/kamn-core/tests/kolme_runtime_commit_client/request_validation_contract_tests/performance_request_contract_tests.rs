use super::*;

#[test]
fn performance_runtime_commit_contract_lane_stays_within_budget() {
    let mut client =
        InMemoryKolmeRuntimeCommitClient::new("kolme-local").expect("client should build");
    let started = Instant::now();
    submit_performance_requests(&mut client);
    assert_budget(started.elapsed().as_millis());
}

fn submit_performance_requests(client: &mut InMemoryKolmeRuntimeCommitClient) {
    for nonce in 1..=256 {
        let request = build_performance_request(nonce);
        assert_submitted(
            client
                .submit_commit(&request)
                .expect("submit should succeed"),
        );
    }
}

fn build_performance_request(nonce: u64) -> KolmeRuntimeCommitRequest {
    KolmeRuntimeCommitRequest::deterministic(
        format!("op-sync-perf-{nonce}").as_str(),
        "state:perf",
        "kamn:did:agent:runtime-node-perf",
        nonce,
        format!("payload:perf:{nonce}").as_str(),
    )
    .expect("request should build")
}

fn assert_submitted(outcome: KolmeRuntimeCommitOutcome) {
    assert!(matches!(outcome, KolmeRuntimeCommitOutcome::Submitted(_)));
}

fn assert_budget(elapsed_millis: u128) {
    assert!(
        elapsed_millis < 300,
        "runtime commit contract lane exceeded budget: {elapsed_millis}ms"
    );
}
