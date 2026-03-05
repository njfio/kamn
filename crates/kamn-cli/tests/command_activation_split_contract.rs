use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_command_activation_root_contract_file_stays_within_size_budget() {
    let source = read_repo_file("tests/command_activation_contract.rs");
    let line_count = source.lines().count();
    assert!(
        line_count <= 200,
        "command_activation_contract.rs should stay within 200-line budget after split; got {line_count}"
    );
}

#[test]
fn spec_c02_command_activation_root_contract_file_removes_split_test_blocks() {
    let source = read_repo_file("tests/command_activation_contract.rs");
    for marker in [
        "fn spec_c02_cli_list_messages_command_executes_and_validates_args()",
        "fn spec_c03_cli_verify_proof_command_executes_and_validates_args()",
        "fn spec_c04_cli_task_and_escrow_commands_execute_and_validate_args()",
        "fn spec_c05_cli_core_message_and_task_commands_execute_and_validate_args()",
        "fn spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args()",
        "fn spec_c08_cli_content_commands_execute_and_validate_args()",
        "fn spec_c09_cli_bridge_commands_execute_and_validate_args()",
    ] {
        assert!(
            !source.contains(marker),
            "command_activation_contract.rs should not keep split test block: {marker}"
        );
    }
}

#[test]
fn spec_c03_command_activation_split_files_exist_and_own_moved_coverage() {
    let core = read_repo_file("tests/command_activation_core_contract.rs");
    let content_bridge = read_repo_file("tests/command_activation_content_bridge_contract.rs");
    let harness = read_repo_file("tests/command_activation_harness.rs");

    for marker in [
        "fn spec_c02_cli_list_messages_command_executes_and_validates_args()",
        "fn spec_c03_cli_verify_proof_command_executes_and_validates_args()",
        "fn spec_c04_cli_task_and_escrow_commands_execute_and_validate_args()",
        "fn spec_c05_cli_core_message_and_task_commands_execute_and_validate_args()",
        "fn spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args()",
    ] {
        assert!(
            core.contains(marker),
            "command_activation_core_contract should keep moved marker: {marker}"
        );
    }

    for marker in [
        "fn spec_c08_cli_content_commands_execute_and_validate_args()",
        "fn spec_c09_cli_bridge_commands_execute_and_validate_args()",
    ] {
        assert!(
            content_bridge.contains(marker),
            "command_activation_content_bridge_contract should keep moved marker: {marker}"
        );
    }

    for marker in [
        "pub(crate) fn reserve_loopback_addr() -> String",
        "pub(crate) fn run_cli_contract_server(",
        "pub(crate) fn parsed(",
        "pub(crate) fn parsed_json(",
    ] {
        assert!(
            harness.contains(marker),
            "command_activation_harness should define shared helper marker: {marker}"
        );
    }
}
