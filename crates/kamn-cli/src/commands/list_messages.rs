use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the list_messages command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let channel_id = required_arg(args, 0, "channel_id")?;
    let handle = connect_handle(args)?;
    let listing = handle.list_messages(channel_id)?;
    Ok(command_output(vec![
        ("channel_id", OutputValue::String(listing.channel_id)),
        ("messages", OutputValue::StringList(listing.messages)),
    ]))
}
