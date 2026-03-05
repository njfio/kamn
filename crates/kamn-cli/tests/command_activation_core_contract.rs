#[path = "command_activation_core_contract/cases.rs"]
mod command_activation_core_cases;
mod command_activation_harness;

use command_activation_harness::{assert_missing_arg_invalid, parsed, with_contract_server};
use kamn_cli::{dispatch, CommandKind};

const ACCEPT_TASK_OUTPUT_LABEL: &str = "accept-task output should include accepted state";
const COMPLETE_TASK_OUTPUT_LABEL: &str = "complete-task output should include completed state";
const FUND_ESCROW_ID_OUTPUT_LABEL: &str = "fund-escrow output should include escrow id";
const FUND_ESCROW_STATE_OUTPUT_LABEL: &str = "fund-escrow output should include funded state";
const RELEASE_ESCROW_OUTPUT_LABEL: &str = "release-escrow output should include released state";
const REGISTER_OUTPUT_LABEL: &str = "register output should include did marker";
const SEND_MESSAGE_OUTPUT_LABEL: &str = "send-message output should include message id";
const CREATE_CHANNEL_OUTPUT_LABEL: &str = "create-channel output should include channel id";
const QUERY_MESSAGE_OUTPUT_LABEL: &str = "query-message output should include status marker";
const CREATE_TASK_OUTPUT_LABEL: &str = "create-task output should include task id";
const QUERY_TASK_ID_OUTPUT_LABEL: &str = "query-task output should include task id";
const QUERY_TASK_STATE_OUTPUT_LABEL: &str = "query-task output should include state projection";
const QUERY_PROFILE_DID_OUTPUT_LABEL: &str = "query-agent-profile output should include did";
const QUERY_PROFILE_SCORE_OUTPUT_LABEL: &str =
    "query-agent-profile output should include reputation_score";

#[test]
fn spec_c02_cli_list_messages_command_executes_and_validates_args() {
    with_contract_server(1, |endpoint| {
        let output = dispatch(&parsed(
            CommandKind::ListMessages,
            endpoint,
            &["channel-cli"],
        ))
        .expect("list-messages should succeed");
        assert!(
            output.text.contains("channel_id=channel-cli"),
            "list-messages output should include channel id: {output:?}"
        );
        assert!(
            output.text.contains("msg-1,msg-2"),
            "list-messages output should include message ids: {output:?}"
        );

        assert_missing_arg_invalid(endpoint, CommandKind::ListMessages, "channel_id");
    });
}

#[test]
fn spec_c03_cli_verify_proof_command_executes_and_validates_args() {
    let output = dispatch(&parsed(
        CommandKind::VerifyProof,
        "http://localhost:18080",
        &["msg-1", "tx-1", "9", "final"],
    ))
    .expect("verify-proof command should succeed");
    assert!(
        output.text.contains("message_id=msg-1"),
        "verify-proof output should include message id: {output:?}"
    );
    assert!(
        output.text.contains("verified=true"),
        "verify-proof output should include verified projection: {output:?}"
    );

    let invalid_block_height = dispatch(&parsed(
        CommandKind::VerifyProof,
        "http://localhost:18080",
        &["msg-1", "tx-1", "not-a-number", "final"],
    ))
    .expect_err("malformed block-height should fail");
    assert!(
        matches!(
            invalid_block_height,
            kamn_agent_lib::AgentLibError::InvalidInput { .. }
        ),
        "verify-proof malformed block-height should be invalid input: {invalid_block_height}"
    );
}

#[test]
fn spec_c04_cli_task_and_escrow_commands_execute_and_validate_args() {
    command_activation_core_cases::run_spec_c04_cli_task_and_escrow_commands_execute_and_validate_args();
}

#[test]
fn spec_c05_cli_core_message_and_task_commands_execute_and_validate_args() {
    command_activation_core_cases::run_spec_c05_cli_core_message_and_task_commands_execute_and_validate_args();
}

#[test]
fn spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args() {
    command_activation_core_cases::run_spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args();
}
