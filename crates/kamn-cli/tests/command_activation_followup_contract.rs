use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_followup_shared_helpers_are_centralized_in_harness() {
    let core = read_repo_file("tests/command_activation_core_contract.rs");
    let content_bridge = read_repo_file("tests/command_activation_content_bridge_contract.rs");
    let harness = read_repo_file("tests/command_activation_harness.rs");

    for source in [core.as_str(), content_bridge.as_str()] {
        for marker in [
            "fn with_contract_server(max_requests: usize, run: impl FnOnce(&str))",
            "fn assert_missing_arg_invalid(endpoint: &str, command: CommandKind, label: &str)",
        ] {
            assert!(
                !source.contains(marker),
                "contract files should not duplicate centralized helper marker: {marker}"
            );
        }
    }

    for marker in [
        "pub(crate) fn with_contract_server(",
        "pub(crate) fn assert_missing_arg_invalid(",
    ] {
        assert!(
            harness.contains(marker),
            "harness should define centralized helper marker: {marker}"
        );
    }
}

#[test]
fn spec_c02_followup_assertions_retain_diagnostic_messages() {
    let core = read_repo_file("tests/command_activation_core_contract.rs");
    let content_bridge = read_repo_file("tests/command_activation_content_bridge_contract.rs");

    for marker in [
        "accept-task output should include accepted state",
        "complete-task output should include completed state",
        "fund-escrow output should include escrow id",
        "fund-escrow output should include funded state",
        "release-escrow output should include released state",
        "register output should include did marker",
        "send-message output should include message id",
        "create-channel output should include channel id",
        "query-message output should include status marker",
        "create-task output should include task id",
        "query-task output should include task id",
        "query-task output should include state projection",
        "query-agent-profile output should include did",
        "query-agent-profile output should include reputation_score",
    ] {
        assert!(
            core.contains(marker),
            "core contract should keep message marker: {marker}"
        );
    }

    for marker in [
        "register-content output should include content id",
        "register-content output should include retention class",
        "expire-content output should include lifecycle state",
        "tombstone-content output should include redaction status",
        "query-content output should include lifecycle state",
        "submit-bridge-message output should include bridge id",
        "submit-bridge-message output should include bridge status",
        "forward-bridge-message output should include bridge status",
        "forward-bridge-message output should include target id",
        "query-bridge-message output should include forward tx marker",
    ] {
        assert!(
            content_bridge.contains(marker),
            "content/bridge contract should keep message marker: {marker}"
        );
    }
}

#[test]
fn spec_c03_followup_harness_dead_code_allowance_is_documented() {
    let harness = read_repo_file("tests/command_activation_harness.rs");
    assert!(
        harness.contains("parsed_json is consumed only by JSON-mode contract suites"),
        "harness should explain why parsed_json keeps dead_code allowance"
    );
}

#[test]
fn spec_c04_followup_spec6373_deviations_capture_extra_files() {
    let spec = read_repo_file("../../specs/6373-kamn-cli-command-activation-contract-split.md");
    assert!(
        !spec.contains("## Deviations\n- None."),
        "6373 spec deviations should not remain 'None' after extra files were introduced"
    );
    for marker in [
        "command_activation_harness_routes.rs",
        "command_activation_split_contract.rs",
    ] {
        assert!(
            spec.contains(marker),
            "6373 deviations should mention extra file marker: {marker}"
        );
    }
}
