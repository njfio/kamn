use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::{AgentLibError, KolmeProofReceipt};

/// Executes the verify_proof command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let message_id = required_arg(args, 0, "message_id")?;
    let tx_hash = required_arg(args, 1, "tx_hash")?;
    let block_height_raw = required_arg(args, 2, "block_height")?;
    let finality = required_arg(args, 3, "finality")?;
    let block_height =
        block_height_raw
            .parse::<u64>()
            .map_err(|_| AgentLibError::InvalidInput {
                field: "block_height",
                reason: "must be an unsigned integer".to_owned(),
            })?;

    let handle = connect_handle(args)?;
    let receipt = KolmeProofReceipt {
        tx_hash: tx_hash.to_owned(),
        block_height,
        finality: finality.to_owned(),
    };
    let verification = handle.verify_proof(message_id, &receipt)?;
    Ok(command_output(vec![
        ("message_id", OutputValue::String(verification.message_id)),
        (
            "block_height",
            OutputValue::Raw(verification.block_height.to_string()),
        ),
        ("finality", OutputValue::String(verification.finality)),
        (
            "verified",
            OutputValue::Raw(verification.verified.to_string()),
        ),
    ]))
}
