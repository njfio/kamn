use kamn_core::{
    build_libp2p_lifecycle_regression_corpus, run_libp2p_lifecycle_regression_case,
    run_libp2p_lifecycle_regression_corpus, PeerLifecycleRegressionExpectedOutcome,
    PeerLifecycleState,
};

#[test]
fn unit_lifecycle_regression_corpus_includes_required_scenarios() {
    let corpus = build_libp2p_lifecycle_regression_corpus();
    let case_ids = corpus
        .iter()
        .map(|case| case.case_id().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        case_ids,
        vec![
            "connect_handshake_disconnect".to_owned(),
            "connect_heartbeat_timeout_recovery".to_owned(),
            "connect_drop_rejoin".to_owned(),
            "invalid_heartbeat_from_disconnected".to_owned(),
        ]
    );
}

#[test]
fn functional_lifecycle_regression_cases_replay_expected_outcomes() {
    let corpus = build_libp2p_lifecycle_regression_corpus();
    for case in &corpus {
        let outcome =
            run_libp2p_lifecycle_regression_case("peer-processor", case).expect("case replays");
        match case.expected_outcome() {
            PeerLifecycleRegressionExpectedOutcome::FinalState(expected) => {
                assert_eq!(outcome.final_state(), Some(*expected));
                assert_eq!(outcome.transition_error_reason_code(), None);
            }
            PeerLifecycleRegressionExpectedOutcome::TransitionError(expected_error) => {
                assert_eq!(outcome.final_state(), None);
                assert_eq!(
                    outcome.transition_error_reason_code(),
                    Some(expected_error.reason_code())
                );
            }
        }
    }
}

#[test]
fn integration_lifecycle_regression_corpus_replay_is_deterministic() {
    let corpus = build_libp2p_lifecycle_regression_corpus();
    let outcomes =
        run_libp2p_lifecycle_regression_corpus("peer-processor", &corpus).expect("corpus replays");

    assert_eq!(outcomes.len(), 4);
    assert_eq!(outcomes[0].case_id(), "connect_handshake_disconnect");
    assert_eq!(
        outcomes[0].final_state(),
        Some(PeerLifecycleState::Disconnected)
    );
    assert_eq!(outcomes[1].case_id(), "connect_heartbeat_timeout_recovery");
    assert_eq!(outcomes[1].final_state(), Some(PeerLifecycleState::Active));
    assert_eq!(outcomes[2].case_id(), "connect_drop_rejoin");
    assert_eq!(outcomes[2].final_state(), Some(PeerLifecycleState::Active));
    assert_eq!(outcomes[3].case_id(), "invalid_heartbeat_from_disconnected");
    assert_eq!(
        outcomes[3].transition_error_reason_code(),
        Some("runtime_peer_transition_invalid")
    );
}

#[test]
fn regression_invalid_transition_case_retains_fail_closed_reason_code() {
    // Regression: #3315
    let corpus = build_libp2p_lifecycle_regression_corpus();
    let invalid_case = corpus
        .iter()
        .find(|case| case.case_id() == "invalid_heartbeat_from_disconnected")
        .expect("invalid transition case should exist");
    let outcome = run_libp2p_lifecycle_regression_case("peer-processor", invalid_case)
        .expect("invalid transition case should replay");

    assert_eq!(outcome.final_state(), None);
    assert_eq!(
        outcome.transition_error_reason_code(),
        Some("runtime_peer_transition_invalid")
    );
}
