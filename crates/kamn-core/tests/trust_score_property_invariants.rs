use kamn_core::{
    AgentReputation, CapabilityVerification, DisputeRecord, Endorsement, ScoreSnapshot,
};

fn generated_reputation() -> AgentReputation {
    AgentReputation {
        agent_did: "kamn:did:agent:generated".to_owned(),
        trust_score: 500,
        delivery_rate: 0.5,
        response_time_avg_ms: 5_000,
        dispute_rate: 0.0,
        tasks_completed: 0,
        tasks_failed: 0,
        tasks_delegated: 0,
        total_earned: 0,
        total_spent: 0,
        endorsements: Vec::new(),
        disputes: Vec::new(),
        verified_capabilities: vec![CapabilityVerification {
            capability: "analysis".to_owned(),
            verifier_did: "kamn:did:agent:verifier-1".to_owned(),
            proof_ref: "ipfs://proof".to_owned(),
            block_height: 10,
        }],
        last_updated_block: 1_000,
        score_history: Vec::new(),
    }
}

fn build_endorsements(count: usize) -> Vec<Endorsement> {
    (0..count)
        .map(|index| Endorsement {
            endorsement_id: format!("endorsement-{index}"),
            from_agent_did: format!("kamn:did:agent:endorser-{index}"),
            note: "generated".to_owned(),
            block_height: 20 + index as u64,
        })
        .collect()
}

#[test]
fn trust_score_property_generated_inputs_stay_within_bounds() {
    let delivery_rates = [0.0, 0.25, 0.5, 0.75, 1.0];
    let dispute_rates = [0.0, 0.1, 0.3, 0.6, 1.0];
    let response_buckets = [500_u64, 1_000, 1_001, 5_000, 5_001, 30_000, 60_000];
    let completion_counts = [0_u64, 1, 10, 25, 100, 500, 1_000, 2_000];
    let endorsement_counts = [0_usize, 1, 5, 25, 50, 75];

    for delivery_rate in delivery_rates {
        for dispute_rate in dispute_rates {
            for response_time in response_buckets {
                for tasks_completed in completion_counts {
                    for endorsements in endorsement_counts {
                        let mut reputation = generated_reputation();
                        reputation.delivery_rate = delivery_rate;
                        reputation.dispute_rate = dispute_rate;
                        reputation.response_time_avg_ms = response_time;
                        reputation.tasks_completed = tasks_completed;
                        reputation.endorsements = build_endorsements(endorsements);

                        let breakdown = kamn_core::calculate_trust_score(&reputation)
                            .expect("generated case should calculate");
                        assert!(breakdown.final_score <= 1_000);
                    }
                }
            }
        }
    }
}

#[test]
fn trust_score_property_dispute_rate_is_non_increasing() {
    let dispute_rates = [0.0, 0.05, 0.1, 0.2, 0.3, 0.5, 0.8, 1.0];
    let mut previous_score = u32::MAX;

    for dispute_rate in dispute_rates {
        let mut reputation = generated_reputation();
        reputation.delivery_rate = 0.9;
        reputation.response_time_avg_ms = 1_000;
        reputation.tasks_completed = 120;
        reputation.endorsements = build_endorsements(5);
        reputation.dispute_rate = dispute_rate;

        let breakdown = kamn_core::calculate_trust_score(&reputation)
            .expect("dispute-rate property case should calculate");
        assert!(
            breakdown.final_score <= previous_score,
            "score must not increase as dispute rate rises: {previous_score} -> {}",
            breakdown.final_score
        );
        previous_score = breakdown.final_score;
    }
}

#[test]
fn trust_score_regression_stale_only_history_must_not_improve_decay_multiplier() {
    // Regression: #768
    let mut previous_multiplier = u16::MAX;

    for stale_count in 1_u64..=8 {
        let mut reputation = generated_reputation();
        reputation.tasks_completed = 100;
        reputation.score_history = (0..stale_count)
            .map(|index| ScoreSnapshot {
                trust_score: 500,
                block_height: 100 - index,
            })
            .collect::<Vec<_>>();
        reputation.disputes = (0..stale_count.min(5))
            .map(|index| DisputeRecord {
                dispute_id: format!("dispute-{index}"),
                opened_by: "kamn:did:agent:requester".to_owned(),
                reason: "generated".to_owned(),
                block_height: 300 + index,
            })
            .collect::<Vec<_>>();

        let breakdown = kamn_core::calculate_trust_score(&reputation)
            .expect("stale-history property case should calculate");
        assert!(
            breakdown.decay_multiplier_bps <= previous_multiplier,
            "stale-only snapshots must not increase decay multiplier: {previous_multiplier} -> {}",
            breakdown.decay_multiplier_bps
        );
        previous_multiplier = breakdown.decay_multiplier_bps;
    }
}
