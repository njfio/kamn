mod command_activation_harness;

use command_activation_harness::{parsed, reserve_loopback_addr, run_cli_contract_server, wait_for_server_ready};
use kamn_agent_lib::AgentLibError;
use kamn_cli::{dispatch, CommandKind};
use std::thread;

fn with_contract_server(max_requests: usize, run: impl FnOnce(&str)) {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_cli_contract_server(server_addr, max_requests));
    wait_for_server_ready();
    let endpoint = format!("http://{bind_addr}");

    run(endpoint.as_str());

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

fn assert_invalid_input(error: AgentLibError, label: &str) {
    assert!(
        matches!(error, AgentLibError::InvalidInput { .. }),
        "missing arg for {label} should be invalid input: {error}"
    );
}

fn assert_missing_arg_invalid(endpoint: &str, command: CommandKind, label: &str) {
    let error = dispatch(&parsed(command, endpoint, &[])).expect_err("missing required arg should fail");
    assert_invalid_input(error, label);
}

#[test]
fn spec_c02_cli_list_messages_command_executes_and_validates_args() {
    with_contract_server(1, |endpoint| {
        let output = dispatch(&parsed(CommandKind::ListMessages, endpoint, &["channel-cli"]))
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
    assert_invalid_input(invalid_block_height, "verify_proof_block_height");
}

#[test]
fn spec_c04_cli_task_and_escrow_commands_execute_and_validate_args() {
    with_contract_server(4, |endpoint| {
        let accept_output = dispatch(&parsed(CommandKind::AcceptTask, endpoint, &["task-cli"]))
            .expect("accept-task should succeed");
        assert!(accept_output.text.contains("state=accepted"));

        let complete_output = dispatch(&parsed(CommandKind::CompleteTask, endpoint, &["task-cli"]))
            .expect("complete-task should succeed");
        assert!(complete_output.text.contains("state=completed"));

        let fund_output = dispatch(&parsed(
            CommandKind::FundEscrow,
            endpoint,
            &[r#"{"task_id":"task-cli","amount":100}"#],
        ))
        .expect("fund-escrow should succeed");
        assert!(fund_output.text.contains("escrow_id=escrow-cli"));
        assert!(fund_output.text.contains("state=funded"));

        let release_output = dispatch(&parsed(CommandKind::ReleaseEscrow, endpoint, &["escrow-cli"]))
            .expect("release-escrow should succeed");
        assert!(release_output.text.contains("state=released"));

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

#[test]
fn spec_c05_cli_core_message_and_task_commands_execute_and_validate_args() {
    with_contract_server(4, |endpoint| {
        let register_output =
            dispatch(&parsed(CommandKind::Register, endpoint, &[])).expect("register should succeed");
        assert!(register_output.text.contains("kamn:did:agent"));

        let send_output = dispatch(&parsed(
            CommandKind::SendMessage,
            endpoint,
            &[r#"{"message":"hello"}"#],
        ))
        .expect("send-message should succeed");
        assert!(send_output.text.contains("message_id=msg-cli"));

        let channel_output = dispatch(&parsed(
            CommandKind::CreateChannel,
            endpoint,
            &[r#"{"name":"ops"}"#],
        ))
        .expect("create-channel should succeed");
        assert!(channel_output.text.contains("channel_id=channel-cli"));

        let query_output = dispatch(&parsed(CommandKind::QueryMessage, endpoint, &["msg-cli"]))
            .expect("query-message should succeed");
        assert!(query_output.text.contains("status=created"));

        let task_output = dispatch(&parsed(CommandKind::CreateTask, endpoint, &[r#"{"task":"triage"}"#]))
            .expect("create-task should succeed");
        assert!(task_output.text.contains("task_id=task-cli"));

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

#[test]
fn spec_c07_cli_query_task_and_profile_commands_execute_and_validate_args() {
    with_contract_server(2, |endpoint| {
        let query_task_output = dispatch(&parsed(CommandKind::QueryTask, endpoint, &["task-cli"]))
            .expect("query-task should succeed");
        assert!(query_task_output.text.contains("task_id=task-cli"));
        assert!(query_task_output.text.contains("state=submitted"));

        let query_profile_output = dispatch(&parsed(
            CommandKind::QueryAgentProfile,
            endpoint,
            &["kamn:did:agent:alice"],
        ))
        .expect("query-agent-profile should succeed");
        assert!(query_profile_output.text.contains("did=kamn:did:agent:alice"));
        assert!(query_profile_output.text.contains("reputation_score=777"));

        for (command, label) in [
            (CommandKind::QueryTask, "query_task_id"),
            (CommandKind::QueryAgentProfile, "query_agent_profile_did"),
        ] {
            assert_missing_arg_invalid(endpoint, command, label);
        }
    });
}
