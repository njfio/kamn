use super::{
    probe_binary_invocation_with_status_runner, probe_command_args_for_label,
    should_retry_text_file_busy, PhaseResultStatus, ETXTBSY_ERRNO, TEXT_FILE_BUSY_RETRY_LIMIT,
};

#[test]
fn unit_should_retry_text_file_busy_accepts_etxtbsy_within_retry_budget() {
    let busy_error = std::io::Error::from_raw_os_error(ETXTBSY_ERRNO);
    assert!(
        should_retry_text_file_busy(&busy_error, 0),
        "first ETXTBSY spawn error should retry"
    );
    assert!(
        should_retry_text_file_busy(&busy_error, TEXT_FILE_BUSY_RETRY_LIMIT - 1),
        "last in-budget ETXTBSY spawn error should retry"
    );
}

#[test]
fn unit_should_retry_text_file_busy_rejects_non_retryable_error_shapes() {
    let busy_error = std::io::Error::from_raw_os_error(ETXTBSY_ERRNO);
    assert!(
        !should_retry_text_file_busy(&busy_error, TEXT_FILE_BUSY_RETRY_LIMIT),
        "ETXTBSY should not retry after budget exhaustion"
    );

    let missing_binary_error = std::io::Error::from_raw_os_error(2);
    assert!(
        !should_retry_text_file_busy(&missing_binary_error, 0),
        "non-ETXTBSY spawn errors must fail immediately"
    );
}

#[test]
fn unit_probe_binary_invocation_retries_text_file_busy_up_to_retry_limit() {
    let mut calls = 0usize;
    let (status, detail) = busy_retry_result(&mut calls);
    assert_retry_budget_respected(calls);
    assert_fail_closed_probe_result(status, detail);
}

#[test]
fn unit_probe_binary_invocation_fails_immediately_for_non_retryable_spawn_errors() {
    let mut calls = 0usize;
    let (status, detail) = probe_binary_invocation_with_status_runner("kolme", || {
        calls += 1;
        Err(std::io::Error::from_raw_os_error(2))
    });
    assert_eq!(
        status,
        PhaseResultStatus::Fail,
        "non-retryable spawn errors should fail immediately"
    );
    assert_eq!(calls, 1, "non-retryable spawn errors should not loop");
    assert!(
        detail.contains("kolme probe failed"),
        "failure detail should retain probe context: {detail}"
    );
}

#[test]
fn unit_probe_command_args_for_kamn_components_use_role_startup_shape() {
    assert_eq!(
        probe_command_args_for_label("kamn_processor"),
        ["--role", "processor"],
        "processor probe should use deterministic role startup args"
    );
    assert_eq!(
        probe_command_args_for_label("kamn_listener"),
        ["--role", "listener"],
        "listener probe should use deterministic role startup args"
    );
    assert_eq!(
        probe_command_args_for_label("kamn_approver"),
        ["--role", "approver"],
        "approver probe should use deterministic role startup args"
    );
}

#[test]
fn unit_probe_command_args_for_non_kamn_components_use_help_surface() {
    assert_eq!(
        probe_command_args_for_label("kolme"),
        ["--help"],
        "kolme probe should continue using help command shape"
    );
    assert_eq!(
        probe_command_args_for_label("agent"),
        ["--help"],
        "agent probe should continue using help command shape"
    );
}

fn busy_retry_result(calls: &mut usize) -> (PhaseResultStatus, String) {
    probe_binary_invocation_with_status_runner("kolme", || {
        *calls += 1;
        assert_retry_loop_in_budget(*calls);
        Err(std::io::Error::from_raw_os_error(ETXTBSY_ERRNO))
    })
}

fn assert_retry_loop_in_budget(calls: usize) {
    assert!(
        calls <= TEXT_FILE_BUSY_RETRY_LIMIT + 1,
        "retry loop exceeded ETXTBSY budget: calls={calls}"
    );
}

fn assert_retry_budget_respected(calls: usize) {
    assert_eq!(
        calls,
        TEXT_FILE_BUSY_RETRY_LIMIT + 1,
        "expected initial call plus bounded retries"
    );
}

fn assert_fail_closed_probe_result(status: PhaseResultStatus, detail: String) {
    assert_eq!(
        status,
        PhaseResultStatus::Fail,
        "exhausted ETXTBSY retries should fail closed"
    );
    assert!(
        detail.contains("kolme probe failed"),
        "failure detail should retain probe context: {detail}"
    );
    assert!(
        !detail.contains("retry budget exhausted"),
        "expected concrete spawn error once retry budget is consumed: {detail}"
    );
}
