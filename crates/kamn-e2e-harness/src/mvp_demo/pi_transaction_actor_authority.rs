use super::pi_transaction_actor_model::{authority_error, is_sha256, Actor, ServiceReceipt};

struct ReceiptContract {
    tool: &'static str,
    action: &'static str,
    resource: Resource,
    state: &'static str,
}

enum Resource {
    Task,
    Escrow,
}

pub(super) fn validate_authority(actor: &Actor) -> Result<(), String> {
    if !is_sha256(&actor.service_profile_commitment) {
        return Err(authority_error());
    }
    let contracts = contracts(actor.actor.as_str())?;
    if contracts.len() != actor.service_receipts.len() {
        return Err(authority_error());
    }
    for (receipt, contract) in actor.service_receipts.iter().zip(contracts) {
        validate_receipt(actor, receipt, &contract)?;
    }
    require_unique_receipts(&actor.service_receipts)
}

fn validate_receipt(
    actor: &Actor,
    receipt: &ServiceReceipt,
    contract: &ReceiptContract,
) -> Result<(), String> {
    let resource = match contract.resource {
        Resource::Task => actor.task_id.as_str(),
        Resource::Escrow => actor.escrow_id.as_str(),
    };
    let valid = receipt.actor_did == actor.did
        && receipt.tool == contract.tool
        && receipt.action == contract.action
        && receipt.resource_id == resource
        && receipt.resulting_state == contract.state
        && !receipt.service_receipt_id.is_empty()
        && is_sha256(&receipt.service_receipt_digest);
    valid.then_some(()).ok_or_else(authority_error)
}

fn require_unique_receipts(receipts: &[ServiceReceipt]) -> Result<(), String> {
    for (index, receipt) in receipts.iter().enumerate() {
        if receipts[index + 1..]
            .iter()
            .any(|other| other.service_receipt_id == receipt.service_receipt_id)
        {
            return Err(authority_error());
        }
    }
    Ok(())
}

fn contracts(role: &str) -> Result<Vec<ReceiptContract>, String> {
    match role {
        "agent_a" => Ok(agent_a_contracts()),
        "agent_b" => Ok(agent_b_contracts()),
        "agent_c" => Ok(Vec::new()),
        _ => Err("PI_ACTOR_IDENTITY_INVALID".to_owned()),
    }
}

fn agent_a_contracts() -> Vec<ReceiptContract> {
    vec![
        contract("create_task", "task:create", Resource::Task, "submitted"),
        contract("fund_escrow", "escrow:fund", Resource::Escrow, "funded"),
        contract(
            "release_escrow",
            "escrow:release-authorize",
            Resource::Escrow,
            "release-authorized",
        ),
        contract(
            "release_escrow",
            "settlement:confirmed",
            Resource::Escrow,
            "confirmed",
        ),
    ]
}

fn agent_b_contracts() -> Vec<ReceiptContract> {
    vec![
        contract("accept_task", "task:accept", Resource::Task, "accepted"),
        contract(
            "complete_task",
            "task:complete",
            Resource::Task,
            "completed",
        ),
    ]
}

fn contract(
    tool: &'static str,
    action: &'static str,
    resource: Resource,
    state: &'static str,
) -> ReceiptContract {
    ReceiptContract {
        tool,
        action,
        resource,
        state,
    }
}
