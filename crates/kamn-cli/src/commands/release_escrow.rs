use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the release_escrow command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let escrow_id = required_arg(args, 0, "escrow_id")?;
    let handle = connect_handle(args)?;
    let receipt = handle.release_escrow(escrow_id)?;
    Ok(format!(
        "escrow_id={} state={}",
        receipt.escrow_id, receipt.state
    ))
}
