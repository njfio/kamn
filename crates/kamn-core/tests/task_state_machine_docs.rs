const DOC: &str = include_str!("../../../docs/foundation/task-state-machine.md");

#[test]
fn doc_contains_task_lifecycle_scope_and_transition_map() {
    assert!(DOC.contains("# Task State Machine and Transition Validator"));
    assert!(DOC.contains("TaskLifecycle::new(task_id)"));
    assert!(DOC.contains("## Supported Transitions"));
}

#[test]
fn doc_contains_transition_evidence_reason_code_contract() {
    assert!(DOC.contains("## Transition Evidence and Reason-Code Contract"));
    assert!(DOC.contains("transition_with_evidence(TaskTransition)"));
    assert!(DOC.contains("TaskTransitionEvidence"));
    assert!(DOC.contains("task_transition_allowed"));
    assert!(DOC.contains("task_transition_invalid_edge"));
    assert!(DOC.contains("task_transition_terminal_state"));
    assert!(DOC.contains("task_history_invalid"));
    assert!(DOC.contains("task_id_empty"));
}

#[test]
fn doc_includes_transition_contract_validation_commands() {
    assert!(DOC.contains("cargo test -p kamn-core --test task_state_machine"));
    assert!(DOC.contains("cargo test -p kamn-core --test task_escrow_transition_contracts"));
    assert!(DOC.contains("cargo test -p kamn-core --test task_state_machine_docs"));
}

#[test]
fn regression_marker_for_transition_reason_code_drift_is_present() {
    // Regression: #903
    assert!(DOC.contains(
        "transition reason-code drift and illegal transition acceptance fail closed (`Regression: #903`)."
    ));
}
