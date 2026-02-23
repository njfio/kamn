use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the submit_bridge_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let payload = required_arg(args, 0, "submit_bridge_message_payload")?;
    let handle = connect_handle(args)?;
    let submission = handle.submit_bridge_message(payload)?;
    Ok(command_output(vec![
        ("bridge_id", OutputValue::String(submission.bridge_id)),
        (
            "source_message_id",
            OutputValue::String(submission.source_message_id),
        ),
        (
            "bridge_status",
            OutputValue::String(submission.bridge_status),
        ),
    ]))
}
