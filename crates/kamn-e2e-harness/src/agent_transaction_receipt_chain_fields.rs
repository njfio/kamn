use super::agent_transaction_receipt_chain_digest as digest;
use super::agent_transaction_receipt_chain_model::{EscrowReceipt, TaskReceipt};

pub(super) enum Mutation<'a> {
    Task(&'a TaskReceipt),
    Escrow(&'a EscrowReceipt),
}

pub(super) struct Fields<'a> {
    pub(super) receipt_id: &'a str,
    pub(super) receipt_digest: String,
    pub(super) authorization_action: &'a str,
    pub(super) authorization_resource: String,
    pub(super) action: &'a str,
    pub(super) resource_id: &'a str,
    pub(super) correlation_id: &'a str,
    pub(super) idempotency_key: &'a str,
    pub(super) prior_state: &'a str,
    pub(super) resulting_state: &'a str,
}

impl Mutation<'_> {
    pub(super) fn fields(&self) -> Fields<'_> {
        match self {
            Self::Task(receipt) => task_fields(receipt),
            Self::Escrow(receipt) => escrow_fields(receipt),
        }
    }
}

fn task_fields(receipt: &TaskReceipt) -> Fields<'_> {
    let authorization_resource = if receipt.action == "task:create" {
        "transaction:new".to_owned()
    } else {
        format!("task:{}", receipt.task_id)
    };
    Fields {
        receipt_id: &receipt.receipt_id,
        receipt_digest: digest::task(receipt),
        authorization_action: &receipt.action,
        authorization_resource,
        action: &receipt.action,
        resource_id: &receipt.task_id,
        correlation_id: &receipt.correlation_id,
        idempotency_key: &receipt.idempotency_key,
        prior_state: &receipt.prior_state,
        resulting_state: &receipt.resulting_state,
    }
}

fn escrow_fields(receipt: &EscrowReceipt) -> Fields<'_> {
    let release = receipt.action == "escrow:release-authorize";
    Fields {
        receipt_id: &receipt.receipt_id,
        receipt_digest: digest::escrow(receipt),
        authorization_action: if release {
            "escrow:release"
        } else {
            &receipt.action
        },
        authorization_resource: if release {
            format!("escrow:{}", receipt.escrow_id)
        } else {
            format!("task:{}", receipt.task_id)
        },
        action: &receipt.action,
        resource_id: &receipt.escrow_id,
        correlation_id: &receipt.correlation_id,
        idempotency_key: &receipt.idempotency_key,
        prior_state: &receipt.prior_state,
        resulting_state: &receipt.resulting_state,
    }
}
