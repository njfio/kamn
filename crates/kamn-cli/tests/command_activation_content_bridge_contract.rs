mod command_activation_harness;

use command_activation_harness::{
    assert_missing_arg_invalid, assert_output_contains, parsed, with_contract_server,
};
use kamn_cli::{dispatch, CommandKind};

#[test]
fn spec_c08_cli_content_commands_execute_and_validate_args() {
    with_contract_server(4, |endpoint| {
        let register_output = dispatch(&parsed(
            CommandKind::RegisterContent,
            endpoint,
            &[r#"{"content":"abc","retention_class":"standard"}"#],
        ))
        .expect("register-content should succeed");
        assert_output_contains(
            register_output.text.as_str(),
            "content_id=content-cli",
            "register-content output should include content id",
        );
        assert_output_contains(
            register_output.text.as_str(),
            "retention_class=standard",
            "register-content output should include retention class",
        );

        let expire_output = dispatch(&parsed(
            CommandKind::ExpireContent,
            endpoint,
            &["content-cli"],
        ))
        .expect("expire-content should succeed");
        assert_output_contains(
            expire_output.text.as_str(),
            "lifecycle_state=expired",
            "expire-content output should include lifecycle state",
        );

        let tombstone_output = dispatch(&parsed(
            CommandKind::TombstoneContent,
            endpoint,
            &["content-cli"],
        ))
        .expect("tombstone-content should succeed");
        assert_output_contains(
            tombstone_output.text.as_str(),
            "redaction_status=redacted",
            "tombstone-content output should include redaction status",
        );

        let query_output = dispatch(&parsed(
            CommandKind::QueryContent,
            endpoint,
            &["content-cli"],
        ))
        .expect("query-content should succeed");
        assert_output_contains(
            query_output.text.as_str(),
            "lifecycle_state=tombstoned",
            "query-content output should include lifecycle state",
        );

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
        assert_output_contains(
            submit_output.text.as_str(),
            "bridge_id=bridge-cli",
            "submit-bridge-message output should include bridge id",
        );
        assert_output_contains(
            submit_output.text.as_str(),
            "bridge_status=submitted",
            "submit-bridge-message output should include bridge status",
        );

        let forward_output = dispatch(&parsed(
            CommandKind::ForwardBridgeMessage,
            endpoint,
            &["bridge-cli"],
        ))
        .expect("forward-bridge-message should succeed");
        assert_output_contains(
            forward_output.text.as_str(),
            "bridge_status=forwarded",
            "forward-bridge-message output should include bridge status",
        );
        assert_output_contains(
            forward_output.text.as_str(),
            "target_message_id=msg-bridge-target-cli",
            "forward-bridge-message output should include target id",
        );

        let query_output = dispatch(&parsed(
            CommandKind::QueryBridgeMessage,
            endpoint,
            &["bridge-cli"],
        ))
        .expect("query-bridge-message should succeed");
        assert_output_contains(
            query_output.text.as_str(),
            "forward_tx_hash=sha256:bridge-forwarded-cli",
            "query-bridge-message output should include forward tx marker",
        );

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
