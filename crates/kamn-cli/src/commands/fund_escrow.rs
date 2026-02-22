use crate::commands::{connect_handle, required_arg};
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the fund_escrow command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let payload = required_arg(args, 0, "fund_escrow_payload")?;
    let handle = connect_handle(args)?;
    let receipt = handle.fund_escrow(payload)?;
    Ok(format!(
        "escrow_id={} state={}",
        receipt.escrow_id, receipt.state
    ))
}
