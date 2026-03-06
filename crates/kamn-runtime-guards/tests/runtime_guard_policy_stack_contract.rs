const POLICY_STACK_TESTS: &str = include_str!("runtime_guard_policy_stack.rs");

#[test]
fn contract_policy_stack_keeps_invalid_input_error_propagation_regression() {
    assert!(
        POLICY_STACK_TESTS.contains(
            "integration_policy_stack_propagates_invalid_input_error_from_anti_spam_engine"
        ),
        "runtime_guard_policy_stack.rs must keep the invalid-input error propagation regression"
    );
}
