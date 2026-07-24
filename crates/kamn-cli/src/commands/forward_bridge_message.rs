use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::AgentLibError;

/// Executes the forward_bridge_message command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let bridge_id = required_arg(args, 0, "forward_bridge_message_id")?;
    let handle = connect_handle(args)?;
    let status = handle.forward_bridge_message(bridge_id)?;
    let mut fields = vec![
        ("bridge_id", OutputValue::String(status.bridge_id)),
        ("bridge_status", OutputValue::String(status.bridge_status)),
        (
            "target_message_id",
            OutputValue::String(status.target_message_id),
        ),
        (
            "forward_tx_hash",
            OutputValue::String(status.forward_tx_hash),
        ),
    ];
    if let Some(receipt) = status.bridge_receipt {
        fields.extend([
            ("receipt_id", OutputValue::String(receipt.receipt_id)),
            (
                "bridge_receipt_digest",
                OutputValue::String(receipt.receipt_digest),
            ),
            (
                "transaction_signature",
                OutputValue::String(receipt.transaction_signature),
            ),
            (
                "finalized_slot",
                OutputValue::Raw(receipt.finalized_slot.to_string()),
            ),
        ]);
    }
    Ok(command_output(fields))
}
