use crate::commands::connect_handle;
use crate::ParsedCliArgs;
use kamn_agent_lib::AgentLibError;

/// Executes the register command.
pub fn execute(args: &ParsedCliArgs) -> Result<String, AgentLibError> {
    let handle = connect_handle(args)?;
    Ok(format!("did={}", handle.identity().did().as_str()))
}
