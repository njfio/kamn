use super::command_activation_harness::{
    assert_missing_arg_invalid, assert_output_contains, parsed, with_contract_server,
};
use super::{
    ACCEPT_TASK_OUTPUT_LABEL, COMPLETE_TASK_OUTPUT_LABEL, CREATE_CHANNEL_OUTPUT_LABEL,
    CREATE_TASK_OUTPUT_LABEL, FUND_ESCROW_ID_OUTPUT_LABEL, FUND_ESCROW_STATE_OUTPUT_LABEL,
    QUERY_MESSAGE_OUTPUT_LABEL, QUERY_PROFILE_DID_OUTPUT_LABEL, QUERY_PROFILE_SCORE_OUTPUT_LABEL,
    QUERY_TASK_ID_OUTPUT_LABEL, QUERY_TASK_STATE_OUTPUT_LABEL, REGISTER_OUTPUT_LABEL,
    RELEASE_ESCROW_OUTPUT_LABEL, SEND_MESSAGE_OUTPUT_LABEL,
};
use kamn_cli::{dispatch, CommandKind};

macro_rules! assert_contains {
    ($output:expr, $expected:expr, $label:expr) => {
        assert_output_contains($output.text.as_str(), $expected, $label);
    };
}

pub(super) fn run_spec_c04_cli_task_and_escrow_commands_execute_and_validate_args() {
    with_contract_server(4, |endpoint| {
        let accept_output = dispatch(&parsed(CommandKind::AcceptTask, endpoint, &["task-cli"]))
            .expect("accept-task should succeed");
        assert_contains!(accept_output, "state=accepted", ACCEPT_TASK_OUTPUT_LABEL);

        let complete_output = dispatch(&parsed(CommandKind::CompleteTask, endpoint, &["task-cli"]))
            .expect("complete-task should succeed");
        assert_contains!(
            complete_output,
            "state=completed",
            COMPLETE_TASK_OUTPUT_LABEL
        );

        let fund_output = dispatch(&parsed(
            CommandKind::FundEscrow,
            endpoint,
            &[r#"{"task_id":"task-cli","amount":100}"#],
        ))
        .expect("fund-escrow should succeed");
        assert_contains!(
            fund_output,
            "escrow_id=escrow-cli",
            FUND_ESCROW_ID_OUTPUT_LABEL
        );
        assert_contains!(fund_output, "state=funded", FUND_ESCROW_STATE_OUTPUT_LABEL);

        let release_output = dispatch(&parsed(
            CommandKind::ReleaseEscrow,
            endpoint,
            &["escrow-cli"],
        ))
        .expect("release-escrow should succeed");
        assert_contains!(
            release_output,
            "state=released",
            RELEASE_ESCROW_OUTPUT_LABEL
        );

        for (command, label) in [
            (CommandKind::AcceptTask, "task_id"),
            (CommandKind::CompleteTask, "task_id"),
            (CommandKind::FundEscrow, "fund_escrow_payload"),
            (CommandKind::ReleaseEscrow, "escrow_id"),
        ] {
            assert_missing_arg_invalid(endpoint, command, label);
        }
    });
}

pub(super) fn run_spec_c05_cli_core_message_and_task_commands_execute_and_validate_args() {
    with_contract_server(4, |endpoint| {
        let register_output = dispatch(&parsed(CommandKind::Register, endpoint, &[]))
            .expect("register should succeed");
        assert_contains!(register_output, "kamn:did:agent", REGISTER_OUTPUT_LABEL);

        let send_output = dispatch(&parsed(
            CommandKind::SendMessage,
            endpoint,
            &[r#"{"message":"hello"}"#],
        ))
        .expect("send-message should succeed");
        assert_contains!(send_output, "message_id=msg-cli", SEND_MESSAGE_OUTPUT_LABEL);

        let channel_output = dispatch(&parsed(
            CommandKind::CreateChannel,
            endpoint,
            &[r#"{"name":"ops"}"#],
        ))
        .expect("create-channel should succeed");
        assert_contains!(
            channel_output,
            "channel_id=channel-cli",
            CREATE_CHANNEL_OUTPUT_LABEL
        );

        let query_output = dispatch(&parsed(CommandKind::QueryMessage, endpoint, &["msg-cli"]))
            .expect("query-message should succeed");
        assert_contains!(query_output, "status=created", QUERY_MESSAGE_OUTPUT_LABEL);

        let task_output = dispatch(&parsed(
            CommandKind::CreateTask,
            endpoint,
            &[r#"{"task":"triage"}"#],
        ))
        .expect("create-task should succeed");
        assert_contains!(task_output, "task_id=task-cli", CREATE_TASK_OUTPUT_LABEL);

        for (command, label) in [
            (CommandKind::SendMessage, "send_message_payload"),
            (CommandKind::CreateChannel, "create_channel_payload"),
            (CommandKind::QueryMessage, "query_message_id"),
            (CommandKind::CreateTask, "create_task_payload"),
        ] {
            assert_missing_arg_invalid(endpoint, command, label);
        }
    });
}

pub(super) fn run_spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args() {
    with_contract_server(2, |endpoint| {
        let query_task_output = dispatch(&parsed(CommandKind::QueryTask, endpoint, &["task-cli"]))
            .expect("query-task should succeed");
        assert_contains!(
            query_task_output,
            "task_id=task-cli",
            QUERY_TASK_ID_OUTPUT_LABEL
        );
        assert_contains!(
            query_task_output,
            "state=submitted",
            QUERY_TASK_STATE_OUTPUT_LABEL
        );

        let query_profile_output = dispatch(&parsed(
            CommandKind::QueryAgentProfile,
            endpoint,
            &["kamn:did:agent:alice"],
        ))
        .expect("query-agent-profile should succeed");
        assert_contains!(
            query_profile_output,
            "did=kamn:did:agent:alice",
            QUERY_PROFILE_DID_OUTPUT_LABEL
        );
        assert_contains!(
            query_profile_output,
            "reputation_score=777",
            QUERY_PROFILE_SCORE_OUTPUT_LABEL
        );

        for (command, label) in [
            (CommandKind::QueryTask, "query_task_id"),
            (CommandKind::QueryAgentProfile, "query_agent_profile_did"),
        ] {
            assert_missing_arg_invalid(endpoint, command, label);
        }
    });
}
