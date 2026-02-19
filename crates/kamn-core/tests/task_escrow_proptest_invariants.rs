#[path = "task_escrow_proptest_invariants/escrow_domain.rs"]
mod escrow_domain;
#[path = "task_escrow_proptest_invariants/shared.rs"]
mod shared;
#[path = "task_escrow_proptest_invariants/task_domain.rs"]
mod task_domain;

#[test]
fn unit_task_escrow_proptest_budget_envelope_is_bounded() {
    assert!(
        shared::TASK_CASES <= 256,
        "task case budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        shared::ESCROW_CASES <= 256,
        "escrow case budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        shared::MAX_SEQUENCE_LEN <= 32,
        "transition sequence budget must stay bounded for deterministic CI runtime"
    );
}
