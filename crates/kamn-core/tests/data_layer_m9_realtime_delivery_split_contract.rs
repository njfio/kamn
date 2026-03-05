use std::fs;

const CONTROLS_BACKPRESSURE_ROOT_MARKERS: [&str; 9] = [
    "#[path = \"data_layer_m9_realtime_delivery/controls_backpressure_cases.rs\"]",
    "mod controls_backpressure_cases;",
    "controls_backpressure_cases::run_spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes();",
    "controls_backpressure_cases::run_spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts();",
    "controls_backpressure_cases::run_spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions();",
    "controls_backpressure_cases::run_spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input();",
    "fn spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes()",
    "fn spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts()",
    "fn spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions()",
];

const CONTROLS_BACKPRESSURE_CASES_MARKERS: [&str; 4] = [
    "pub(super) fn run_spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes(",
    "pub(super) fn run_spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts(",
    "pub(super) fn run_spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions(",
    "pub(super) fn run_spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input(",
];

const BASELINE_FLOW_ROOT_MARKERS: [&str; 8] = [
    "#[path = \"data_layer_m9_realtime_delivery/baseline_flow_cases.rs\"]",
    "mod baseline_flow_cases;",
    "baseline_flow_cases::run_spec_c01_connected_recipient_without_backlog_receives_delivered_ack();",
    "baseline_flow_cases::run_spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered();",
    "baseline_flow_cases::run_spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers();",
    "baseline_flow_cases::run_spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed();",
    "baseline_flow_cases::run_spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter();",
    "fn spec_c01_connected_recipient_without_backlog_receives_delivered_ack()",
];

const BASELINE_FLOW_CASES_MARKERS: [&str; 5] = [
    "pub(super) fn run_spec_c01_connected_recipient_without_backlog_receives_delivered_ack(",
    "pub(super) fn run_spec_c02_presence_query_is_denied_until_relationship_linkage_is_registered(",
    "pub(super) fn run_spec_c03_backpressure_thresholds_emit_warning_and_sustained_escalation_markers(",
    "pub(super) fn run_spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed(",
    "pub(super) fn run_spec_c05_queue_full_dispatch_keeps_pending_cap_and_increments_deferred_counter(",
];

const QUEUE_CHANNEL_ROOT_MARKERS: [&str; 8] = [
    "#[path = \"data_layer_m9_realtime_delivery/queue_channel_cases.rs\"]",
    "mod queue_channel_cases;",
    "queue_channel_cases::run_spec_c06_queue_snapshot_preserves_pending_dispatch_order();",
    "queue_channel_cases::run_spec_c07_queue_snapshot_preserves_deferred_dispatch_order();",
    "queue_channel_cases::run_spec_c08_duplicate_message_identifier_is_rejected_fail_closed();",
    "queue_channel_cases::run_spec_c09_channel_dispatch_requires_sender_and_recipient_membership();",
    "fn spec_c06_queue_snapshot_preserves_pending_dispatch_order()",
    "fn spec_c09_channel_dispatch_requires_sender_and_recipient_membership()",
];

const QUEUE_CHANNEL_CASES_MARKERS: [&str; 4] = [
    "pub(super) fn run_spec_c06_queue_snapshot_preserves_pending_dispatch_order(",
    "pub(super) fn run_spec_c07_queue_snapshot_preserves_deferred_dispatch_order(",
    "pub(super) fn run_spec_c08_duplicate_message_identifier_is_rejected_fail_closed(",
    "pub(super) fn run_spec_c09_channel_dispatch_requires_sender_and_recipient_membership(",
];

const INPUT_VALIDATION_ROOT_MARKERS: [&str; 7] = [
    "#[path = \"data_layer_m9_realtime_delivery/input_validation_cases.rs\"]",
    "mod input_validation_cases;",
    "input_validation_cases::run_spec_c14_invalid_requester_owner_did_fails_closed_with_field_taxonomy();",
    "input_validation_cases::run_spec_c15_invalid_sender_and_recipient_agent_dids_fail_closed_with_field_taxonomy();",
    "input_validation_cases::run_spec_c16_invalid_presence_requester_agent_did_fails_closed_with_field_taxonomy();",
    "fn spec_c14_invalid_requester_owner_did_fails_closed_with_field_taxonomy()",
    "fn spec_c16_invalid_presence_requester_agent_did_fails_closed_with_field_taxonomy()",
];

const INPUT_VALIDATION_CASES_MARKERS: [&str; 3] = [
    "pub(super) fn run_spec_c14_invalid_requester_owner_did_fails_closed_with_field_taxonomy(",
    "pub(super) fn run_spec_c15_invalid_sender_and_recipient_agent_dids_fail_closed_with_field_taxonomy(",
    "pub(super) fn run_spec_c16_invalid_presence_requester_agent_did_fails_closed_with_field_taxonomy(",
];

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_controls_backpressure_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m9_realtime_delivery.rs");
    let cases =
        read_repo_file("tests/data_layer_m9_realtime_delivery/controls_backpressure_cases.rs");

    for marker in CONTROLS_BACKPRESSURE_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root realtime-delivery contract should contain controls/backpressure delegation marker: {marker}"
        );
    }

    for marker in CONTROLS_BACKPRESSURE_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "controls/backpressure cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c02_baseline_flow_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m9_realtime_delivery.rs");
    let cases = read_repo_file("tests/data_layer_m9_realtime_delivery/baseline_flow_cases.rs");

    for marker in BASELINE_FLOW_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root realtime-delivery contract should contain baseline-flow delegation marker: {marker}"
        );
    }

    for marker in BASELINE_FLOW_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "baseline-flow cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c03_queue_channel_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m9_realtime_delivery.rs");
    let cases = read_repo_file("tests/data_layer_m9_realtime_delivery/queue_channel_cases.rs");

    for marker in QUEUE_CHANNEL_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root realtime-delivery contract should contain queue-channel delegation marker: {marker}"
        );
    }

    for marker in QUEUE_CHANNEL_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "queue-channel cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c04_input_validation_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m9_realtime_delivery.rs");
    let cases = read_repo_file("tests/data_layer_m9_realtime_delivery/input_validation_cases.rs");

    for marker in INPUT_VALIDATION_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root realtime-delivery contract should contain input-validation delegation marker: {marker}"
        );
    }

    for marker in INPUT_VALIDATION_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "input-validation cases module should define marker: {marker}"
        );
    }
}
