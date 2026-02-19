#[path = "property_invariant_helpers.rs"]
mod property_invariant_helpers;

use kamn_core::{PeerLifecycleEvent, PeerLifecycleState, TaskState};
use proptest::test_runner::RngSeed;

const RUNTIME_STATE_MODEL_DOC: &str =
    include_str!("../../../docs/architecture/runtime-state-model.md");

#[test]
fn unit_property_helper_seed_override_parser_is_deterministic() {
    assert_eq!(
        property_invariant_helpers::parse_seed_override(None, 17),
        17
    );
    assert_eq!(
        property_invariant_helpers::parse_seed_override(Some("  "), 17),
        17
    );
    assert_eq!(
        property_invariant_helpers::parse_seed_override(Some("123456"), 17),
        123456
    );
    assert_eq!(
        property_invariant_helpers::parse_seed_override(Some("0x0000002a"), 17),
        42
    );
    assert_eq!(
        property_invariant_helpers::parse_seed_override(Some("not-a-number"), 17),
        17
    );
}

#[test]
fn unit_property_helper_transition_legality_matches_expected_edges() {
    assert!(property_invariant_helpers::is_legal_task_state_step(
        TaskState::Submitted,
        TaskState::Accepted
    ));
    assert!(!property_invariant_helpers::is_legal_task_state_step(
        TaskState::Submitted,
        TaskState::Completed
    ));

    assert_eq!(
        property_invariant_helpers::expected_peer_next_state(
            PeerLifecycleState::Disconnected,
            PeerLifecycleEvent::StartConnect
        ),
        Some(PeerLifecycleState::Connecting)
    );
    assert_eq!(
        property_invariant_helpers::expected_peer_next_state(
            PeerLifecycleState::Active,
            PeerLifecycleEvent::StartConnect
        ),
        None
    );
}

#[test]
fn integration_property_helper_config_uses_seed_and_source_file_contracts() {
    let config = property_invariant_helpers::deterministic_proptest_config(13, 99, file!());
    assert_eq!(config.cases, 13);
    assert_eq!(config.rng_seed, RngSeed::Fixed(99));
    assert!(config.failure_persistence.is_some());
    assert_eq!(config.source_file, Some(file!()));
}

#[test]
fn regression_runtime_state_model_doc_references_property_helper_library() {
    assert!(RUNTIME_STATE_MODEL_DOC.contains("property_invariant_helpers.rs"));
    assert!(RUNTIME_STATE_MODEL_DOC.contains("parse_seed_override"));
    assert!(RUNTIME_STATE_MODEL_DOC.contains("deterministic_proptest_config"));
    assert!(RUNTIME_STATE_MODEL_DOC.contains("is_legal_task_state_step"));
    assert!(RUNTIME_STATE_MODEL_DOC.contains("expected_peer_next_state"));
}
