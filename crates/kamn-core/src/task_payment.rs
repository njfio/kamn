use crate::{
    AgentDid, EscrowLifecycle, EscrowLifecycleError, TaskOperationEngine, TaskOperationError,
    TaskState,
};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentOffer {
    pub task_id: String,
    pub escrow_id: String,
    pub payer_did: String,
    pub payee_did: String,
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentConfirm {
    pub task_id: String,
    pub escrow_id: String,
    pub confirmer_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingOffer {
    offer: PaymentOffer,
    confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskPaymentWorkflow {
    offers_by_task: BTreeMap<String, PendingOffer>,
}

impl TaskPaymentWorkflow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn submit_offer(
        &mut self,
        offer: PaymentOffer,
        tasks: &TaskOperationEngine,
        escrow: &EscrowLifecycle,
    ) -> Result<(), TaskPaymentError> {
        validate_offer_shape(&offer)?;
        validate_did(&offer.payer_did)?;
        validate_did(&offer.payee_did)?;

        if self.offers_by_task.contains_key(&offer.task_id) {
            return Err(TaskPaymentError::DuplicateOffer(offer.task_id));
        }

        let task = tasks
            .task(&offer.task_id)
            .map_err(|error| TaskPaymentError::TaskLookup(error.to_string()))?;
        let state = task.lifecycle.state();
        if state != TaskState::Completed {
            return Err(TaskPaymentError::TaskNotCompleted {
                task_id: offer.task_id,
                state,
            });
        }
        validate_offer_participants(&offer, &task.requester, task.assignee.as_deref())?;

        let remaining = escrow.remaining_amount();
        if offer.amount > remaining {
            return Err(TaskPaymentError::OfferExceedsEscrow {
                offered: offer.amount,
                remaining,
            });
        }

        self.offers_by_task.insert(
            offer.task_id.clone(),
            PendingOffer {
                offer,
                confirmed: false,
            },
        );
        Ok(())
    }

    pub fn confirm_offer(
        &mut self,
        confirm: PaymentConfirm,
        escrow: &mut EscrowLifecycle,
    ) -> Result<(), TaskPaymentError> {
        if confirm.task_id.trim().is_empty() {
            return Err(TaskPaymentError::EmptyField("task_id"));
        }
        if confirm.escrow_id.trim().is_empty() {
            return Err(TaskPaymentError::EmptyField("escrow_id"));
        }
        validate_did(&confirm.confirmer_did)?;

        let pending = self
            .offers_by_task
            .get_mut(&confirm.task_id)
            .ok_or_else(|| TaskPaymentError::UnknownOffer(confirm.task_id.clone()))?;
        if pending.confirmed {
            return Err(TaskPaymentError::DuplicateConfirm(confirm.task_id));
        }
        if pending.offer.escrow_id != confirm.escrow_id {
            return Err(TaskPaymentError::EscrowMismatch {
                expected: pending.offer.escrow_id.clone(),
                found: confirm.escrow_id,
            });
        }
        if pending.offer.payer_did != confirm.confirmer_did {
            return Err(TaskPaymentError::UnauthorizedConfirmer {
                expected: pending.offer.payer_did.clone(),
                found: confirm.confirmer_did,
            });
        }

        escrow
            .release(pending.offer.amount)
            .map_err(|error| TaskPaymentError::Escrow(error.to_string()))?;
        pending.confirmed = true;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskPaymentError {
    DuplicateConfirm(String),
    DuplicateOffer(String),
    EmptyField(&'static str),
    Escrow(String),
    EscrowMismatch { expected: String, found: String },
    InvalidDid(String),
    InvalidOfferAmount(u128),
    PayerRequesterMismatch { expected: String, found: String },
    PayeeAssigneeMismatch { expected: String, found: String },
    OfferExceedsEscrow { offered: u128, remaining: u128 },
    TaskLookup(String),
    TaskMissingAssignee(String),
    TaskNotCompleted { task_id: String, state: TaskState },
    UnauthorizedConfirmer { expected: String, found: String },
    UnknownOffer(String),
}

impl fmt::Display for TaskPaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateConfirm(task_id) => {
                write!(f, "payment confirm already processed for task {task_id}")
            }
            Self::DuplicateOffer(task_id) => {
                write!(f, "payment offer already registered for task {task_id}")
            }
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::Escrow(error) => write!(f, "escrow error: {error}"),
            Self::EscrowMismatch { expected, found } => write!(
                f,
                "escrow reference mismatch, expected {expected}, found {found}"
            ),
            Self::InvalidDid(error) => write!(f, "invalid did: {error}"),
            Self::InvalidOfferAmount(amount) => {
                write!(f, "offer amount must be greater than zero, found {amount}")
            }
            Self::PayerRequesterMismatch { expected, found } => write!(
                f,
                "payer DID must match task requester, expected {expected}, found {found}"
            ),
            Self::PayeeAssigneeMismatch { expected, found } => write!(
                f,
                "payee DID must match task assignee, expected {expected}, found {found}"
            ),
            Self::OfferExceedsEscrow { offered, remaining } => write!(
                f,
                "offer amount exceeds escrow remaining balance, offered {offered}, remaining {remaining}"
            ),
            Self::TaskLookup(error) => write!(f, "task lookup failed: {error}"),
            Self::TaskMissingAssignee(task_id) => {
                write!(f, "task {task_id} has no assignee for payment offer")
            }
            Self::TaskNotCompleted { task_id, state } => {
                write!(
                    f,
                    "task {task_id} must be completed before payment offer, found state {state:?}"
                )
            }
            Self::UnauthorizedConfirmer { expected, found } => write!(
                f,
                "unauthorized confirmer, expected {expected}, found {found}"
            ),
            Self::UnknownOffer(task_id) => write!(f, "unknown payment offer for task {task_id}"),
        }
    }
}

impl std::error::Error for TaskPaymentError {}

fn validate_offer_shape(offer: &PaymentOffer) -> Result<(), TaskPaymentError> {
    if offer.task_id.trim().is_empty() {
        return Err(TaskPaymentError::EmptyField("task_id"));
    }
    if offer.escrow_id.trim().is_empty() {
        return Err(TaskPaymentError::EmptyField("escrow_id"));
    }
    if offer.amount == 0 {
        return Err(TaskPaymentError::InvalidOfferAmount(offer.amount));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), TaskPaymentError> {
    AgentDid::parse(value).map_err(|error| TaskPaymentError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn validate_offer_participants(
    offer: &PaymentOffer,
    task_requester: &str,
    task_assignee: Option<&str>,
) -> Result<(), TaskPaymentError> {
    if offer.payer_did != task_requester {
        return Err(TaskPaymentError::PayerRequesterMismatch {
            expected: task_requester.to_owned(),
            found: offer.payer_did.clone(),
        });
    }

    let assignee = task_assignee
        .ok_or_else(|| TaskPaymentError::TaskMissingAssignee(offer.task_id.clone()))?;
    if offer.payee_did != assignee {
        return Err(TaskPaymentError::PayeeAssigneeMismatch {
            expected: assignee.to_owned(),
            found: offer.payee_did.clone(),
        });
    }

    Ok(())
}

impl From<TaskOperationError> for TaskPaymentError {
    fn from(error: TaskOperationError) -> Self {
        Self::TaskLookup(error.to_string())
    }
}

impl From<EscrowLifecycleError> for TaskPaymentError {
    fn from(error: EscrowLifecycleError) -> Self {
        Self::Escrow(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{validate_offer_participants, PaymentOffer, TaskPaymentError};

    fn offer(payer: &str, payee: &str) -> PaymentOffer {
        PaymentOffer {
            task_id: "task-pay-unit".to_owned(),
            escrow_id: "escrow-unit".to_owned(),
            payer_did: payer.to_owned(),
            payee_did: payee.to_owned(),
            amount: 10,
        }
    }

    #[test]
    fn participant_check_rejects_payer_requester_mismatch() {
        assert_eq!(
            validate_offer_participants(
                &offer("kamn:did:agent:observer-9", "kamn:did:agent:worker-1"),
                "kamn:did:agent:requester-1",
                Some("kamn:did:agent:worker-1")
            ),
            Err(TaskPaymentError::PayerRequesterMismatch {
                expected: "kamn:did:agent:requester-1".to_owned(),
                found: "kamn:did:agent:observer-9".to_owned(),
            })
        );
    }

    #[test]
    fn participant_check_rejects_payee_assignee_mismatch() {
        assert_eq!(
            validate_offer_participants(
                &offer("kamn:did:agent:requester-1", "kamn:did:agent:observer-9"),
                "kamn:did:agent:requester-1",
                Some("kamn:did:agent:worker-1")
            ),
            Err(TaskPaymentError::PayeeAssigneeMismatch {
                expected: "kamn:did:agent:worker-1".to_owned(),
                found: "kamn:did:agent:observer-9".to_owned(),
            })
        );
    }

    #[test]
    fn participant_check_requires_task_assignee() {
        assert_eq!(
            validate_offer_participants(
                &offer("kamn:did:agent:requester-1", "kamn:did:agent:worker-1"),
                "kamn:did:agent:requester-1",
                None
            ),
            Err(TaskPaymentError::TaskMissingAssignee(
                "task-pay-unit".to_owned()
            ))
        );
    }
}
