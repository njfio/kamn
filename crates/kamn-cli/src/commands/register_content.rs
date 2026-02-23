use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the register_content command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let payload = required_arg(args, 0, "register_content_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.register_content(payload)?;
    Ok(command_output(vec![
        ("content_id", OutputValue::String(receipt.content_id)),
        (
            "retention_class",
            OutputValue::String(receipt.retention_class),
        ),
        (
            "lifecycle_state",
            OutputValue::String(receipt.lifecycle_state),
        ),
        (
            "redaction_status",
            OutputValue::String(receipt.redaction_status),
        ),
    ]))
}
