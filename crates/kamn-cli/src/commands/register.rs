use crate::commands::connect_handle;
use crate::commands::{command_output, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the register command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let handle = connect_handle(args)?;
    Ok(command_output(vec![(
        "did",
        OutputValue::String(handle.identity().did().as_str().to_owned()),
    )]))
}
