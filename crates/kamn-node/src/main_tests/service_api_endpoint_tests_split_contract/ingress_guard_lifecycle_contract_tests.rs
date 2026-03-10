use super::support::*;

#[test]
fn spec_c37_service_api_endpoint_root_file_removes_moved_ingress_guard_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded()",
        "fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension()",
        "fn integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic()",
        "fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded()",
        "fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code()",
        "fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender()",
        "fn integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection()",
        "fn regression_service_api_endpoint_returns_timeout_error_when_no_requests_arrive()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved ingress-guard marker: {marker}"
        );
    }
}

#[test]
fn spec_c38_service_api_endpoint_ingress_guard_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(INGRESS_GUARD_LIFECYCLE_MODULE_FILE);
    let ingress_budget = read_repo_file(INGRESS_BUDGET_FILE);
    let sender_anti_spam = read_repo_file(SENDER_ANTI_SPAM_FILE);
    let replay_guard = read_repo_file(REPLAY_GUARD_FILE);
    let concurrency_guard = read_repo_file(CONCURRENCY_GUARD_FILE);
    let lifecycle_projection = read_repo_file(LIFECYCLE_PROJECTION_FILE);

    assert_ingress_guard_module_declarations(module_source.as_str());
    assert_ingress_guard_markers(
        ingress_budget.as_str(),
        sender_anti_spam.as_str(),
        replay_guard.as_str(),
        concurrency_guard.as_str(),
        lifecycle_projection.as_str(),
    );
}

fn assert_ingress_guard_module_declarations(module_source: &str) {
    for marker in [
        "mod ingress_budget_contract_tests;",
        "mod sender_anti_spam_contract_tests;",
        "mod replay_guard_contract_tests;",
        "mod concurrency_guard_contract_tests;",
        "mod lifecycle_projection_contract_tests;",
    ] {
        assert!(
            module_source.contains(marker),
            "ingress_guard_lifecycle_contract_tests.rs should declare submodule marker: {marker}"
        );
    }
}

fn assert_ingress_guard_markers(
    ingress_budget: &str,
    sender_anti_spam: &str,
    replay_guard: &str,
    concurrency_guard: &str,
    lifecycle_projection: &str,
) {
    assert_ingress_budget_markers(ingress_budget);
    assert_sender_anti_spam_markers(sender_anti_spam);
    assert_replay_guard_markers(replay_guard);
    assert_concurrency_guard_markers(concurrency_guard);
    assert_lifecycle_projection_markers(lifecycle_projection);
}

fn assert_ingress_budget_markers(source: &str) {
    assert_ingress_guard_file_markers(
        source,
        &[
            "fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded()",
            "fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code()",
            "fn regression_service_api_endpoint_unauthorized_ingress_consumes_request_budget()",
            "fn regression_service_api_endpoint_returns_timeout_error_when_no_requests_arrive()",
        ],
        "ingress budget contract file",
    );
}

fn assert_sender_anti_spam_markers(source: &str) {
    assert_ingress_guard_file_markers(
        source,
        &[
            "fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension()",
            "fn integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic()",
        ],
        "sender anti-spam contract file",
    );
}

fn assert_replay_guard_markers(source: &str) {
    assert_ingress_guard_file_markers(
        source,
        &[
            "fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender()",
            "fn integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement()",
            "fn regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable()",
        ],
        "replay guard contract file",
    );
}

fn assert_concurrency_guard_markers(source: &str) {
    assert_ingress_guard_file_markers(
        source,
        &[
            "fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded()",
            "fn integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts()",
            "fn regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds()",
        ],
        "concurrency guard contract file",
    );
}

fn assert_lifecycle_projection_markers(source: &str) {
    assert_ingress_guard_file_markers(
        source,
        &[
            "fn unit_service_api_endpoint_lifecycle_rejection_projection_is_deterministic()",
            "fn functional_service_api_endpoint_lifecycle_rejection_projection_maps_limiter_classes()",
            "fn functional_service_api_endpoint_backpressure_projection_covers_reason_codes()",
            "fn integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection()",
            "fn regression_service_api_endpoint_lifecycle_projection_sender_suspension_class_stays_stable()",
            "fn performance_service_api_endpoint_lifecycle_projection_loop_stays_within_local_budget()",
        ],
        "lifecycle projection contract file",
    );
}

fn assert_ingress_guard_file_markers(source: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            source.contains(marker),
            "{label} should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c39_service_api_endpoint_root_declares_ingress_guard_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod ingress_guard_lifecycle_contract_tests;"),
        "service_api_endpoint_tests.rs should declare ingress-guard-lifecycle submodule"
    );
}

#[test]
fn spec_c40_service_api_endpoint_ingress_guard_split_files_stay_below_budget() {
    for path in [
        INGRESS_GUARD_LIFECYCLE_MODULE_FILE,
        INGRESS_BUDGET_FILE,
        SENDER_ANTI_SPAM_FILE,
        REPLAY_GUARD_FILE,
        CONCURRENCY_GUARD_FILE,
        LIFECYCLE_PROJECTION_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
