#[path = "task_escrow_proptest_invariants/escrow_domain.rs"]
mod escrow_domain;
#[path = "task_escrow_proptest_invariants/shared.rs"]
mod shared;
#[path = "task_escrow_proptest_invariants/task_domain.rs"]
mod task_domain;

#[test]
fn unit_task_escrow_proptest_budget_envelope_is_bounded() {
    let task_cases = std::hint::black_box(shared::TASK_CASES);
    let escrow_cases = std::hint::black_box(shared::ESCROW_CASES);
    let max_sequence_len = std::hint::black_box(shared::MAX_SEQUENCE_LEN);

    assert!(
        task_cases <= 256,
        "task case budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        escrow_cases <= 256,
        "escrow case budget must stay bounded for deterministic CI runtime"
    );
    assert!(
        max_sequence_len <= 32,
        "transition sequence budget must stay bounded for deterministic CI runtime"
    );
}
