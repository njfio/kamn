const ROOT_SUITE: &str = include_str!("../task_escrow_proptest_invariants.rs");
const TASK_DOMAIN_SUITE: &str = include_str!("../task_escrow_proptest_invariants/task_domain.rs");
const ESCROW_DOMAIN_SUITE: &str =
    include_str!("../task_escrow_proptest_invariants/escrow_domain.rs");
const SHARED_SUITE: &str = include_str!("../task_escrow_proptest_invariants/shared.rs");

#[test]
fn regression_task_escrow_suite_discovery_markers_remain_stable() {
    assert!(ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/shared.rs\"]"));
    assert!(ROOT_SUITE.contains("mod shared;"));
    assert!(ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/task_domain.rs\"]"));
    assert!(ROOT_SUITE.contains("mod task_domain;"));
    assert!(ROOT_SUITE.contains("#[path = \"task_escrow_proptest_invariants/escrow_domain.rs\"]"));
    assert!(ROOT_SUITE.contains("mod escrow_domain;"));

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
    assert!(SHARED_SUITE.contains("pub const TASK_CASES: u32 = 192;"));
    assert!(SHARED_SUITE.contains("pub const ESCROW_CASES: u32 = 192;"));
    assert!(SHARED_SUITE.contains("pub const MAX_SEQUENCE_LEN: usize = 32;"));
    assert!(SHARED_SUITE
        .contains("pub const TASK_SEED_ENV_KEY: &str = \"KAMN_PROPTEST_TASK_ESCROW_SEED\";"));
    assert!(SHARED_SUITE
        .contains("pub const ESCROW_SEED_ENV_KEY: &str = \"KAMN_PROPTEST_ESCROW_SEED\";"));
}
