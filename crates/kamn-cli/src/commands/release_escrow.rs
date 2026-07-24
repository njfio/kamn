use crate::commands::{command_output, connect_handle, required_arg, OutputValue};
use crate::{CommandOutput, ParsedCliArgs};
use kamn_agent_lib::{AgentLibError, ServiceAuthoritativeSettlement};

/// Executes the release_escrow command.
pub fn execute(args: &ParsedCliArgs) -> Result<CommandOutput, AgentLibError> {
    let escrow_id = required_arg(args, 0, "escrow_id")?;
    let handle = connect_handle(args)?;
    let receipt = match args.passthrough.get(1) {
        Some(idempotency) => handle.release_escrow_with_payload(
            escrow_id,
            release_payload(idempotency, args.passthrough.get(2)).as_str(),
        )?,
        None => handle.release_escrow(escrow_id)?,
    };
    let mut fields = vec![
        ("escrow_id", OutputValue::String(receipt.escrow_id)),
        ("state", OutputValue::String(receipt.state)),
    ];
    if let Some(authority) = receipt.authoritative_settlement {
        fields.extend(authority_fields(authority));
    }
    Ok(command_output(fields))
}

fn release_payload(idempotency: &str, bridge_id: Option<&String>) -> String {
    match bridge_id {
        Some(bridge_id) => serde_json::json!({
            "idempotency_key": idempotency,
            "authority_mode": "bridge-receipt",
            "bridge_id": bridge_id,
        })
        .to_string(),
        None => serde_json::json!({"idempotency_key": idempotency}).to_string(),
    }
}

fn authority_fields(value: ServiceAuthoritativeSettlement) -> Vec<(&'static str, OutputValue)> {
    vec![
        ("bridge_id", OutputValue::String(value.bridge_id)),
        (
            "bridge_receipt_id",
            OutputValue::String(value.bridge_receipt_id),
        ),
        (
            "bridge_receipt_digest",
            OutputValue::String(value.bridge_receipt_digest),
        ),
        (
            "settlement_receipt_id",
            OutputValue::String(value.settlement_receipt_id),
        ),
        (
            "settlement_receipt_digest",
            OutputValue::String(value.settlement_receipt_digest),
        ),
        ("action", OutputValue::String(value.action)),
        ("resource_id", OutputValue::String(value.resource_id)),
        ("actor_did", OutputValue::String(value.actor_did)),
        (
            "resulting_state",
            OutputValue::String(value.resulting_state),
        ),
        ("task_id", OutputValue::String(value.task_id)),
        ("recipient", OutputValue::String(value.recipient)),
        (
            "amount_lamports",
            OutputValue::Raw(value.amount_lamports.to_string()),
        ),
        ("asset", OutputValue::String(value.asset)),
        ("network", OutputValue::String(value.network)),
        (
            "transaction_signature",
            OutputValue::String(value.transaction_signature),
        ),
        ("commitment", OutputValue::String(value.commitment)),
        (
            "finalized_slot",
            OutputValue::Raw(value.finalized_slot.to_string()),
        ),
        (
            "receipt_chain_commitment",
            OutputValue::String(value.receipt_chain_commitment),
        ),
        ("terms_digest", OutputValue::String(value.terms_digest)),
        (
            "idempotency_key",
            OutputValue::String(value.idempotency_key),
        ),
    ]
}
