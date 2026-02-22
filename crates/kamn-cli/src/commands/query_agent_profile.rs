use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the query_agent_profile command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let did = required_arg(args, 0, "query_agent_profile_did")?;
    let handle = connect_handle(args)?;
    let profile = handle.query_agent_profile(did)?;
    Ok(command_output(vec![
        ("did", OutputValue::String(profile.did)),
        (
            "reputation_score",
            OutputValue::String(profile.reputation_score.to_string()),
        ),
    ]))
}
