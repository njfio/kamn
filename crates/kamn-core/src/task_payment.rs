use crate::{
    AgentDid, EscrowLifecycle, EscrowLifecycleError, TaskOperationEngine, TaskOperationError,
    TaskState,
};
use std::collections::BTreeMap;
use std::fmt;

const TASK_PAYMENT_INVALID_PAYER_DID_REASON_CODE: &str = "task_payment_invalid_payer_did";
const TASK_PAYMENT_INVALID_PAYEE_DID_REASON_CODE: &str = "task_payment_invalid_payee_did";
const TASK_PAYMENT_INVALID_CONFIRMER_DID_REASON_CODE: &str = "task_payment_invalid_confirmer_did";

/// Payment offer submitted for a completed task.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentOffer {
    /// Task identifier that the offer settles.
    pub task_id: String,
    /// Escrow identifier funding the payout.
    pub escrow_id: String,
    /// Requester DID funding the payment.
    pub payer_did: String,
    /// Assignee DID receiving the payment.
    pub payee_did: String,
    /// Amount to release from escrow in atomic units.
    pub amount: u128,
}

/// Confirmation payload authorizing release for a payment offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentConfirm {
    /// Task identifier for the confirmed offer.
    pub task_id: String,
    /// Escrow identifier tied to the offer.
    pub escrow_id: String,
    /// Payer DID confirming release.
    pub confirmer_did: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingOffer {
    offer: PaymentOffer,
    confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaymentOfferValidated {
    payer_did: AgentDid,
    payee_did: AgentDid,
}

impl TryFrom<&PaymentOffer> for PaymentOfferValidated {
    type Error = TaskPaymentError;

    fn try_from(offer: &PaymentOffer) -> Result<Self, Self::Error> {
        Ok(Self {
            payer_did: parse_agent_did(
                offer.payer_did.as_str(),
                "payer_did",
                TASK_PAYMENT_INVALID_PAYER_DID_REASON_CODE,
            )?,
            payee_did: parse_agent_did(
                offer.payee_did.as_str(),
                "payee_did",
                TASK_PAYMENT_INVALID_PAYEE_DID_REASON_CODE,
            )?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PaymentConfirmValidated {
    confirmer_did: AgentDid,
}

impl TryFrom<&PaymentConfirm> for PaymentConfirmValidated {
    type Error = TaskPaymentError;

    fn try_from(confirm: &PaymentConfirm) -> Result<Self, Self::Error> {
        Ok(Self {
            confirmer_did: parse_agent_did(
                confirm.confirmer_did.as_str(),
                "confirmer_did",
                TASK_PAYMENT_INVALID_CONFIRMER_DID_REASON_CODE,
            )?,
        })
    }
}

/// Task payment workflow that validates and tracks escrow-backed offers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskPaymentWorkflow {
    offers_by_task: BTreeMap<String, PendingOffer>,
}

impl TaskPaymentWorkflow {
    /// Creates an empty task-payment workflow.
    pub fn new() -> Self {
        Self::default()
    }

    /// Submits a payment offer after validating task, escrow, and participant constraints.
    pub fn submit_offer(
        &mut self,
        offer: PaymentOffer,
        tasks: &TaskOperationEngine,
        escrow: &EscrowLifecycle,
    ) -> Result<(), TaskPaymentError> {
        validate_offer_shape(&offer)?;
        let validated_offer = PaymentOfferValidated::try_from(&offer)?;

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
        validate_offer_participants(
            offer.task_id.as_str(),
            validated_offer.payer_did.as_str(),
            validated_offer.payee_did.as_str(),
            task.requester.as_str(),
            task.assignee.as_deref(),
        )?;

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

    /// Confirms a submitted offer and releases escrow funds once.
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
        let validated_confirm = PaymentConfirmValidated::try_from(&confirm)?;

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
                found: validated_confirm.confirmer_did.as_str().to_owned(),
            });
        }

        escrow
            .release(pending.offer.amount)
            .map_err(|error| TaskPaymentError::Escrow(error.to_string()))?;
        pending.confirmed = true;
        Ok(())
    }
}

/// Errors returned by task-payment offer and confirmation workflows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskPaymentError {
    /// Confirmation already exists for the task.
    DuplicateConfirm(String),
    /// Offer already exists for the task.
    DuplicateOffer(String),
    /// Required field is missing.
    EmptyField(&'static str),
    /// Escrow lifecycle call failed.
    Escrow(String),
    /// Escrow identifier does not match the submitted offer.
    EscrowMismatch {
        /// Expected escrow identifier.
        expected: String,
        /// Observed escrow identifier.
        found: String,
    },
    /// DID failed validation.
    InvalidDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Offer amount is invalid.
    InvalidOfferAmount(u128),
    /// Payer DID does not match task requester.
    PayerRequesterMismatch {
        /// Expected requester DID.
        expected: String,
        /// Observed payer DID.
        found: String,
    },
    /// Payee DID does not match task assignee.
    PayeeAssigneeMismatch {
        /// Expected assignee DID.
        expected: String,
        /// Observed payee DID.
        found: String,
    },
    /// Offer amount exceeds remaining escrow balance.
    OfferExceedsEscrow {
        /// Amount offered for release.
        offered: u128,
        /// Remaining escrow balance.
        remaining: u128,
    },
    /// Task lookup failed.
    TaskLookup(String),
    /// Task is missing assignee required for payment routing.
    TaskMissingAssignee(String),
    /// Task is not completed and cannot be paid yet.
    TaskNotCompleted {
        /// Task identifier.
        task_id: String,
        /// Current task state.
        state: TaskState,
    },
    /// Confirmer DID is not authorized to approve release.
    UnauthorizedConfirmer {
        /// Expected confirmer DID.
        expected: String,
        /// Observed confirmer DID.
        found: String,
    },
    /// Offer for the given task does not exist.
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
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
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

impl TaskPaymentError {
    /// Stable reason taxonomy for task payment failures.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::DuplicateConfirm(_) => "task_payment_duplicate_confirm",
            Self::DuplicateOffer(_) => "task_payment_duplicate_offer",
            Self::EmptyField(_) => "task_payment_empty_field",
            Self::Escrow(_) => "task_payment_escrow_error",
            Self::EscrowMismatch { .. } => "task_payment_escrow_mismatch",
            Self::InvalidDid { reason_code, .. } => reason_code,
            Self::InvalidOfferAmount(_) => "task_payment_invalid_offer_amount",
            Self::PayerRequesterMismatch { .. } => "task_payment_payer_requester_mismatch",
            Self::PayeeAssigneeMismatch { .. } => "task_payment_payee_assignee_mismatch",
            Self::OfferExceedsEscrow { .. } => "task_payment_offer_exceeds_escrow",
            Self::TaskLookup(_) => "task_payment_task_lookup_error",
            Self::TaskMissingAssignee(_) => "task_payment_task_missing_assignee",
            Self::TaskNotCompleted { .. } => "task_payment_task_not_completed",
            Self::UnauthorizedConfirmer { .. } => "task_payment_unauthorized_confirmer",
            Self::UnknownOffer(_) => "task_payment_unknown_offer",
        }
    }
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, TaskPaymentError> {
    AgentDid::parse(value).map_err(|error| TaskPaymentError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

fn validate_offer_participants(
    task_id: &str,
    payer_did: &str,
    payee_did: &str,
    task_requester: &str,
    task_assignee: Option<&str>,
) -> Result<(), TaskPaymentError> {
    if payer_did != task_requester {
        return Err(TaskPaymentError::PayerRequesterMismatch {
            expected: task_requester.to_owned(),
            found: payer_did.to_owned(),
        });
    }

    let assignee =
        task_assignee.ok_or_else(|| TaskPaymentError::TaskMissingAssignee(task_id.to_owned()))?;
    if payee_did != assignee {
        return Err(TaskPaymentError::PayeeAssigneeMismatch {
            expected: assignee.to_owned(),
            found: payee_did.to_owned(),
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
    use super::{validate_offer_participants, TaskPaymentError};

    #[test]
    fn participant_check_rejects_payer_requester_mismatch() {
        assert_eq!(
            validate_offer_participants(
                "task-pay-unit",
                "kamn:did:agent:observer-9",
                "kamn:did:agent:worker-1",
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
                "task-pay-unit",
                "kamn:did:agent:requester-1",
                "kamn:did:agent:observer-9",
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
                "task-pay-unit",
                "kamn:did:agent:requester-1",
                "kamn:did:agent:worker-1",
                "kamn:did:agent:requester-1",
                None
            ),
            Err(TaskPaymentError::TaskMissingAssignee(
                "task-pay-unit".to_owned()
            ))
        );
    }
}
