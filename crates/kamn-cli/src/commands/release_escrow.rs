use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the release_escrow command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let escrow_id = required_arg(args, 0, "escrow_id")?;
    let handle = connect_handle(args)?;
    let receipt = handle.release_escrow(escrow_id)?;
    Ok(command_output(vec![
        ("escrow_id", OutputValue::String(receipt.escrow_id)),
        ("state", OutputValue::String(receipt.state)),
    ]))
}
