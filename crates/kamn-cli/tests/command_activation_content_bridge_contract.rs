#[path = "command_activation_content_bridge_contract/cases.rs"]
mod command_activation_content_bridge_cases;
mod command_activation_harness;

const REGISTER_CONTENT_ID_OUTPUT_LABEL: &str = "register-content output should include content id";
const REGISTER_CONTENT_RETENTION_OUTPUT_LABEL: &str =
    "register-content output should include retention class";
const EXPIRE_CONTENT_OUTPUT_LABEL: &str = "expire-content output should include lifecycle state";
const TOMBSTONE_CONTENT_OUTPUT_LABEL: &str =
    "tombstone-content output should include redaction status";
const QUERY_CONTENT_OUTPUT_LABEL: &str = "query-content output should include lifecycle state";
const SUBMIT_BRIDGE_ID_OUTPUT_LABEL: &str =
    "submit-bridge-message output should include bridge id";
const SUBMIT_BRIDGE_STATUS_OUTPUT_LABEL: &str =
    "submit-bridge-message output should include bridge status";
const FORWARD_BRIDGE_STATUS_OUTPUT_LABEL: &str =
    "forward-bridge-message output should include bridge status";
const FORWARD_BRIDGE_TARGET_OUTPUT_LABEL: &str =
    "forward-bridge-message output should include target id";
const QUERY_BRIDGE_OUTPUT_LABEL: &str =
    "query-bridge-message output should include forward tx marker";

#[test]
fn spec_c08_cli_content_commands_execute_and_validate_args() {
    command_activation_content_bridge_cases::run_spec_c08_cli_content_commands_execute_and_validate_args();
}

#[test]
fn spec_c09_cli_bridge_commands_execute_and_validate_args() {
    command_activation_content_bridge_cases::run_spec_c09_cli_bridge_commands_execute_and_validate_args();
}
