use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the fund_escrow command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let payload = required_arg(args, 0, "fund_escrow_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.fund_escrow(payload)?;
    Ok(command_output(vec![
        ("escrow_id", OutputValue::String(receipt.escrow_id)),
        ("state", OutputValue::String(receipt.state)),
    ]))
}
