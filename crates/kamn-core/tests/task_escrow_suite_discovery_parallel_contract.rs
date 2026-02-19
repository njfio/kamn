const ROOT_SUITE: &str = include_str!("task_escrow_proptest_invariants.rs");
const TASK_DOMAIN_SUITE: &str = include_str!("task_escrow_proptest_invariants/task_domain.rs");
const ESCROW_DOMAIN_SUITE: &str = include_str!("task_escrow_proptest_invariants/escrow_domain.rs");

#[path = "task_escrow_proptest_invariants/shared.rs"]
#[allow(dead_code)]
mod shared;

#[test]
fn regression_task_escrow_suite_discovery_markers_remain_stable() {
    assert!(
        ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/shared.rs\"]\nmod shared;")
    );
    assert!(ROOT_SUITE.contains(
        "#[path = \"task_escrow_proptest_invariants/task_domain.rs\"]\nmod task_domain;"
    ));
    assert!(ROOT_SUITE.contains(
        "#[path = \"task_escrow_proptest_invariants/escrow_domain.rs\"]\nmod escrow_domain;"
    ));

    assert!(TASK_DOMAIN_SUITE
        .contains("fn functional_task_lifecycle_proptest_sequence_invariants_hold"));
    assert!(TASK_DOMAIN_SUITE
        .contains("fn functional_task_lifecycle_proptest_transition_evidence_is_legal_and_stable"));
    assert!(TASK_DOMAIN_SUITE
        .contains("fn integration_task_lifecycle_proptest_restore_roundtrip_is_stable"));

    assert!(ESCROW_DOMAIN_SUITE
        .contains("fn integration_escrow_proptest_transition_evidence_preserves_invariants"));
    assert!(ESCROW_DOMAIN_SUITE
        .contains("fn integration_escrow_proptest_conserves_amounts_and_status_projections"));
}

#[test]
fn regression_task_escrow_suite_parallel_boundaries_are_bounded_and_isolated() {
    assert_ne!(
        shared::TASK_SEED_ENV_KEY,
        shared::ESCROW_SEED_ENV_KEY,
        "parallel execution must not share the same seed env var key"
    );
    assert!(
        shared::TASK_CASES <= 256,
        "task proptest case budget must stay bounded"
    );
    assert!(
        shared::ESCROW_CASES <= 256,
        "escrow proptest case budget must stay bounded"
    );
    assert!(
        shared::MAX_SEQUENCE_LEN <= 32,
        "sequence length budget must stay bounded"
    );
}
