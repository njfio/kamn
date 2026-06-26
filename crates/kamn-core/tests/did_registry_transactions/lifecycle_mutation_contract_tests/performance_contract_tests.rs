use super::super::support::*;
use kamn_core::DidLifecycleMutationAction;

#[test]
fn performance_lifecycle_mutation_contract_lane_stays_within_budget() {
    let started = Instant::now();

    for round in 0..64 {
        let mut registry = registry();
        let did = parse_did(format!("kamn:did:agent:lifecycle-perf-{round}").as_str());
        let mut document = document_for(&did, "claude-4");
        set_operator(&mut document, "kamn:did:human:ops-perf");
        registry
            .register(did.clone(), document)
            .expect("register should succeed");
        apply_mutation(
            &mut registry,
            &did,
            "kamn:did:human:ops-perf",
            1,
            DidLifecycleMutationAction::Revoke,
        );
        let mut recovered_document = document_for(&did, "claude-4.1");
        set_operator(&mut recovered_document, "kamn:did:human:ops-perf");
        apply_mutation(
            &mut registry,
            &did,
            "kamn:did:human:ops-perf",
            2,
            DidLifecycleMutationAction::Recover {
                document: recovered_document,
            },
        );
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 800,
        "did lifecycle mutation contract lane exceeded budget: {elapsed_millis}ms"
    );
}
