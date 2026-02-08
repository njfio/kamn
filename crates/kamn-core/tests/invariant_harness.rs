use std::collections::BTreeSet;

use kamn_core::{
    classify_smoke_error, BaselineTransaction, InvariantFailureCode, RoleSmokeNetwork,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scenario {
    Valid,
    DuplicateId,
    NonceSequence,
    StaleStateHash,
    TamperedSignature,
}

fn harness_seed() -> u64 {
    std::env::var("KAMN_INVARIANT_SEED")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(13)
}

fn next_seed(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn scenario_from_seed(value: u64) -> Scenario {
    match value % 5 {
        0 => Scenario::Valid,
        1 => Scenario::DuplicateId,
        2 => Scenario::NonceSequence,
        3 => Scenario::StaleStateHash,
        _ => Scenario::TamperedSignature,
    }
}

fn run_scenario(scenario: Scenario, index: usize) -> Option<InvariantFailureCode> {
    let mut network = RoleSmokeNetwork::new(true);

    match scenario {
        Scenario::Valid => {
            let tx = BaselineTransaction::signed(
                &format!("tx-valid-{index}"),
                "agent-a",
                1,
                "payload-valid",
                network.expected_state_hash(),
            );
            network
                .submit_transaction(tx)
                .expect("valid scenario should submit");
            network
                .produce_block()
                .expect("valid scenario should produce block");
            None
        }
        Scenario::DuplicateId => {
            let tx_id = format!("tx-dup-{index}");
            let first = BaselineTransaction::signed(
                &tx_id,
                "agent-a",
                1,
                "payload-first",
                network.expected_state_hash(),
            );
            network
                .submit_transaction(first)
                .expect("first duplicate scenario transaction should submit");

            let duplicate = BaselineTransaction::signed(
                &tx_id,
                "agent-b",
                1,
                "payload-second",
                network.expected_state_hash(),
            );

            classify_smoke_error(
                &network
                    .submit_transaction(duplicate)
                    .expect_err("duplicate id should fail"),
            )
            .map(|violation| violation.failure_code)
        }
        Scenario::NonceSequence => {
            let tx = BaselineTransaction::signed(
                &format!("tx-nonce-{index}"),
                "agent-a",
                2,
                "payload-nonce",
                network.expected_state_hash(),
            );

            classify_smoke_error(
                &network
                    .submit_transaction(tx)
                    .expect_err("nonce sequence violation should fail"),
            )
            .map(|violation| violation.failure_code)
        }
        Scenario::StaleStateHash => {
            let initial_hash = network.expected_state_hash().to_owned();
            let first = BaselineTransaction::signed(
                &format!("tx-stale-seed-{index}"),
                "agent-a",
                1,
                "payload-first",
                &initial_hash,
            );
            network
                .submit_transaction(first)
                .expect("stale-hash setup should submit first tx");
            network
                .produce_block()
                .expect("stale-hash setup should produce block");

            let stale = BaselineTransaction::signed(
                &format!("tx-stale-{index}"),
                "agent-a",
                2,
                "payload-stale",
                &initial_hash,
            );

            classify_smoke_error(
                &network
                    .submit_transaction(stale)
                    .expect_err("stale state hash should fail"),
            )
            .map(|violation| violation.failure_code)
        }
        Scenario::TamperedSignature => {
            let mut tx = BaselineTransaction::signed(
                &format!("tx-sig-{index}"),
                "agent-a",
                1,
                "payload-sig",
                network.expected_state_hash(),
            );
            tx.signature.push_str("-tampered");

            classify_smoke_error(
                &network
                    .submit_transaction(tx)
                    .expect_err("tampered signature should fail"),
            )
            .map(|violation| violation.failure_code)
        }
    }
}

fn run_seed(seed: u64, rounds: usize) -> Vec<Option<InvariantFailureCode>> {
    let mut state = seed;
    let mut outcomes = Vec::with_capacity(rounds);

    for index in 0..rounds {
        let scenario = scenario_from_seed(next_seed(&mut state));
        outcomes.push(run_scenario(scenario, index));
    }

    outcomes
}

#[test]
fn functional_seeded_harness_is_deterministic() {
    let seed = harness_seed();
    let first = run_seed(seed, 16);
    let second = run_seed(seed, 16);
    assert_eq!(first, second);
}

#[test]
fn integration_seeded_harness_covers_core_negative_invariants() {
    let outcomes = run_seed(harness_seed(), 24);
    let observed: BTreeSet<InvariantFailureCode> = outcomes.into_iter().flatten().collect();

    assert!(observed.contains(&InvariantFailureCode::DuplicateTransactionId));
    assert!(observed.contains(&InvariantFailureCode::NonceOutOfSequence));
    assert!(observed.contains(&InvariantFailureCode::StateHashMismatch));
    assert!(observed.contains(&InvariantFailureCode::InvalidSignature));
}

#[test]
fn regression_seed_97_preserves_stale_hash_coverage() {
    // Regression: #79
    let outcomes = run_seed(97, 24);
    assert!(outcomes
        .into_iter()
        .flatten()
        .any(|code| code == InvariantFailureCode::StateHashMismatch));
}
