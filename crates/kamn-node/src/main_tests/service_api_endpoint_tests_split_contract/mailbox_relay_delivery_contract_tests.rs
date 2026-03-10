use super::support::*;

#[test]
fn spec_c33_service_api_endpoint_root_file_removes_moved_mailbox_relay_delivery_contracts() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract()",
        "fn integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids()",
        "fn integration_service_api_endpoint_cross_node_relay_delivery_contract()",
        "fn integration_service_api_endpoint_rejects_legacy_relay_ingest_dids()",
        "fn regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery()",
        "fn integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered()",
        "fn regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart()",
        "fn integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool()",
    ] {
        assert!(
            !source.contains(marker),
            "service_api_endpoint_tests.rs should not keep moved mailbox/relay marker: {marker}"
        );
    }
}

#[test]
fn spec_c34_service_api_endpoint_mailbox_relay_delivery_module_exists_and_owns_moved_coverage() {
    let module_source = read_repo_file(MAILBOX_RELAY_DELIVERY_MODULE_FILE);
    let recipient_mailbox = read_repo_file(RECIPIENT_MAILBOX_FILE);
    let relay_delivery = read_repo_file(RELAY_DELIVERY_FILE);
    let relay_did_rejection = read_repo_file(RELAY_DID_REJECTION_FILE);
    let relay_status = read_repo_file(RELAY_STATUS_FILE);

    assert_mailbox_relay_module_declarations(module_source.as_str());
    assert_mailbox_relay_markers(
        recipient_mailbox.as_str(),
        relay_delivery.as_str(),
        relay_did_rejection.as_str(),
        relay_status.as_str(),
    );
}

fn assert_mailbox_relay_module_declarations(module_source: &str) {
    for marker in [
        "mod recipient_mailbox_contract_tests;",
        "mod relay_delivery_contract_tests;",
        "mod relay_did_rejection_contract_tests;",
        "mod relay_status_contract_tests;",
        "mod support;",
        "mod state_support;",
    ] {
        assert!(
            module_source.contains(marker),
            "mailbox_relay_delivery_contract_tests.rs should declare submodule marker: {marker}"
        );
    }
}

fn assert_mailbox_relay_markers(
    recipient_mailbox: &str,
    relay_delivery: &str,
    relay_did_rejection: &str,
    relay_status: &str,
) {
    assert_mailbox_relay_delivery_markers(recipient_mailbox, relay_delivery, relay_did_rejection);
    assert_mailbox_relay_status_markers(relay_status);
}

fn assert_mailbox_relay_delivery_markers(
    recipient_mailbox: &str,
    relay_delivery: &str,
    relay_did_rejection: &str,
) {
    assert_recipient_mailbox_markers(recipient_mailbox);
    assert_relay_delivery_markers(relay_delivery);
    assert_relay_did_rejection_markers(relay_did_rejection);
}

fn assert_recipient_mailbox_markers(source: &str) {
    assert_mailbox_relay_file_markers(
        source,
        &["fn integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract()"],
        "recipient mailbox contract file",
    );
}

fn assert_relay_delivery_markers(source: &str) {
    assert_mailbox_relay_file_markers(
        source,
        &[
            "fn integration_service_api_endpoint_cross_node_relay_delivery_contract()",
            "fn integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool()",
        ],
        "relay delivery contract file",
    );
}

fn assert_relay_did_rejection_markers(source: &str) {
    assert_mailbox_relay_file_markers(
        source,
        &[
            "fn integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids()",
            "fn integration_service_api_endpoint_rejects_legacy_relay_ingest_dids()",
        ],
        "relay did rejection contract file",
    );
}

fn assert_mailbox_relay_status_markers(relay_status: &str) {
    assert_mailbox_relay_file_markers(
        relay_status,
        &[
            "fn regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery()",
            "fn integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered()",
            "fn regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart()",
        ],
        "relay status contract file",
    );
}

fn assert_mailbox_relay_file_markers(source: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            source.contains(marker),
            "{label} should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c35_service_api_endpoint_root_declares_mailbox_relay_delivery_submodule() {
    let source = read_repo_file(ROOT_FILE);
    assert!(
        source.contains("mod mailbox_relay_delivery_contract_tests;"),
        "service_api_endpoint_tests.rs should declare mailbox-relay-delivery submodule"
    );
}

#[test]
fn spec_c36_service_api_endpoint_mailbox_relay_delivery_split_files_stay_below_budget() {
    for path in [
        MAILBOX_RELAY_DELIVERY_MODULE_FILE,
        RECIPIENT_MAILBOX_FILE,
        RELAY_DELIVERY_FILE,
        RELAY_DID_REJECTION_FILE,
        RELAY_STATUS_FILE,
        MAILBOX_RELAY_SUPPORT_FILE,
        MAILBOX_RELAY_STATE_SUPPORT_FILE,
    ] {
        let source = read_repo_file(path);
        let line_count = source.lines().count();
        assert!(
            line_count <= 200,
            "{path} should stay below 200 lines after extraction: line_count={line_count}"
        );
    }
}
