mod command_activation_harness;

use command_activation_harness::{
    parsed, reserve_loopback_addr, run_cli_contract_server, wait_for_server_ready,
};
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

fn assert_missing_arg_invalid(endpoint: &str, command: CommandKind, label: &str) {
    let error =
        dispatch(&parsed(command, endpoint, &[])).expect_err("missing required arg should fail");
    assert!(
        matches!(error, AgentLibError::InvalidInput { .. }),
        "missing arg for {label} should be invalid input: {error}"
    );
}

#[test]
fn spec_c08_cli_content_commands_execute_and_validate_args() {
    with_contract_server(4, |endpoint| {
        let register_output = dispatch(&parsed(
            CommandKind::RegisterContent,
            endpoint,
            &[r#"{"content":"abc","retention_class":"standard"}"#],
        ))
        .expect("register-content should succeed");
        assert!(register_output.text.contains("content_id=content-cli"));
        assert!(register_output.text.contains("retention_class=standard"));

        let expire_output = dispatch(&parsed(
            CommandKind::ExpireContent,
            endpoint,
            &["content-cli"],
        ))
        .expect("expire-content should succeed");
        assert!(expire_output.text.contains("lifecycle_state=expired"));

        let tombstone_output = dispatch(&parsed(
            CommandKind::TombstoneContent,
            endpoint,
            &["content-cli"],
        ))
        .expect("tombstone-content should succeed");
        assert!(tombstone_output.text.contains("redaction_status=redacted"));

        let query_output = dispatch(&parsed(
            CommandKind::QueryContent,
            endpoint,
            &["content-cli"],
        ))
        .expect("query-content should succeed");
        assert!(query_output.text.contains("lifecycle_state=tombstoned"));

        for (command, label) in [
            (CommandKind::RegisterContent, "register_content_payload"),
            (CommandKind::ExpireContent, "expire_content_id"),
            (CommandKind::TombstoneContent, "tombstone_content_id"),
            (CommandKind::QueryContent, "query_content_id"),
        ] {
            assert_missing_arg_invalid(endpoint, command, label);
        }
    });
}

#[test]
fn spec_c09_cli_bridge_commands_execute_and_validate_args() {
    with_contract_server(3, |endpoint| {
        let submit_output = dispatch(&parsed(
            CommandKind::SubmitBridgeMessage,
            endpoint,
            &[r#"{"source_message_id":"msg-cli","target_network":"testnet"}"#],
        ))
        .expect("submit-bridge-message should succeed");
        assert!(submit_output.text.contains("bridge_id=bridge-cli"));
        assert!(submit_output.text.contains("bridge_status=submitted"));

        let forward_output = dispatch(&parsed(
            CommandKind::ForwardBridgeMessage,
            endpoint,
            &["bridge-cli"],
        ))
        .expect("forward-bridge-message should succeed");
        assert!(forward_output.text.contains("bridge_status=forwarded"));
        assert!(forward_output
            .text
            .contains("target_message_id=msg-bridge-target-cli"));

        let query_output = dispatch(&parsed(
            CommandKind::QueryBridgeMessage,
            endpoint,
            &["bridge-cli"],
        ))
        .expect("query-bridge-message should succeed");
        assert!(query_output
            .text
            .contains("forward_tx_hash=sha256:bridge-forwarded-cli"));

        for (command, label) in [
            (
                CommandKind::SubmitBridgeMessage,
                "submit_bridge_message_payload",
            ),
            (
                CommandKind::ForwardBridgeMessage,
                "forward_bridge_message_id",
            ),
            (CommandKind::QueryBridgeMessage, "query_bridge_message_id"),
        ] {
            assert_missing_arg_invalid(endpoint, command, label);
        }
    });
}
