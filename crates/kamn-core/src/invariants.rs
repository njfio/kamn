//! Invariant catalog and taxonomy contracts for deterministic guardrail mapping.

use std::fmt;

use crate::smoke::SmokeError;
use crate::transaction::TransactionGuardError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Invariant domain partition used in the canonical catalog.
pub enum InvariantDomain {
    /// Invariants tied to transaction envelope and sequencing rules.
    Transactions,
    /// Invariants tied to state-commit transition rules.
    StateTransitions,
}

impl InvariantDomain {
    /// Returns a stable machine-readable domain label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transactions => "transactions",
            Self::StateTransitions => "state-transitions",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// Stable failure-code taxonomy for invariant violations.
pub enum InvariantFailureCode {
    /// Required field was empty.
    EmptyField,
    /// Nonce value was invalid.
    InvalidNonce,
    /// Nonce value was not sequential for sender.
    NonceOutOfSequence,
    /// Signature verification failed.
    InvalidSignature,
    /// State hash did not match expected value.
    StateHashMismatch,
    /// Transaction identifier was duplicated.
    DuplicateTransactionId,
    /// Unvalidated transaction attempted state commit.
    UnvalidatedCommittedTransaction,
}

impl InvariantFailureCode {
    /// Returns the stable external failure code label.
    pub fn as_code(&self) -> &'static str {
        match self {
            Self::EmptyField => "INV-TX-001-EMPTY-FIELD",
            Self::InvalidNonce => "INV-TX-002-INVALID-NONCE",
            Self::NonceOutOfSequence => "INV-TX-002-NONCE-SEQUENCE",
            Self::InvalidSignature => "INV-TX-003-INVALID-SIGNATURE",
            Self::StateHashMismatch => "INV-TX-004-STATE-HASH-MISMATCH",
            Self::DuplicateTransactionId => "INV-TX-005-DUPLICATE-TX-ID",
            Self::UnvalidatedCommittedTransaction => "INV-TX-006-UNVALIDATED-COMMIT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical invariant specification entry.
pub struct InvariantSpec {
    /// Stable invariant identifier.
    pub id: &'static str,
    /// Invariant domain.
    pub domain: InvariantDomain,
    /// Human-readable invariant description.
    pub description: &'static str,
    /// PRD section references backing the invariant.
    pub prd_refs: &'static [&'static str],
    /// Owning issue identifier.
    pub owner_issue: &'static str,
    /// Failure codes associated with this invariant.
    pub failure_codes: &'static [InvariantFailureCode],
}

const FAILURES_TX_001: &[InvariantFailureCode] = &[InvariantFailureCode::EmptyField];
const FAILURES_TX_002: &[InvariantFailureCode] = &[
    InvariantFailureCode::InvalidNonce,
    InvariantFailureCode::NonceOutOfSequence,
];
const FAILURES_TX_003: &[InvariantFailureCode] = &[InvariantFailureCode::InvalidSignature];
const FAILURES_TX_004: &[InvariantFailureCode] = &[InvariantFailureCode::StateHashMismatch];
const FAILURES_TX_005: &[InvariantFailureCode] = &[InvariantFailureCode::DuplicateTransactionId];
const FAILURES_TX_006: &[InvariantFailureCode] =
    &[InvariantFailureCode::UnvalidatedCommittedTransaction];

/// Canonical sorted invariant catalog used across runtime guardrails.
pub const INVARIANT_CATALOG: &[InvariantSpec] = &[
    InvariantSpec {
        id: "INV-TX-001",
        domain: InvariantDomain::Transactions,
        description: "Transaction envelope fields must be present and non-empty.",
        prd_refs: &["2.2.3", "4.1"],
        owner_issue: "#78",
        failure_codes: FAILURES_TX_001,
    },
    InvariantSpec {
        id: "INV-TX-002",
        domain: InvariantDomain::Transactions,
        description: "Transaction nonce must be positive and per-sender sequential.",
        prd_refs: &["2.2.3", "4.2"],
        owner_issue: "#78",
        failure_codes: FAILURES_TX_002,
    },
    InvariantSpec {
        id: "INV-TX-003",
        domain: InvariantDomain::Transactions,
        description: "Transaction signature must match deterministic signing rules.",
        prd_refs: &["1.3", "2.2.2", "4.1"],
        owner_issue: "#78",
        failure_codes: FAILURES_TX_003,
    },
    InvariantSpec {
        id: "INV-TX-004",
        domain: InvariantDomain::Transactions,
        description: "Transaction state hash must match current expected state hash.",
        prd_refs: &["2.2.3", "13.1"],
        owner_issue: "#78",
        failure_codes: FAILURES_TX_004,
    },
    InvariantSpec {
        id: "INV-TX-005",
        domain: InvariantDomain::Transactions,
        description: "Transaction IDs must be globally unique within observed history.",
        prd_refs: &["2.2.3", "4.2"],
        owner_issue: "#78",
        failure_codes: FAILURES_TX_005,
    },
    InvariantSpec {
        id: "INV-TX-006",
        domain: InvariantDomain::StateTransitions,
        description: "Only validated transactions may participate in state commits.",
        prd_refs: &["2.2.3", "13.1"],
        owner_issue: "#78",
        failure_codes: FAILURES_TX_006,
    },
];

/// Returns the canonical invariant catalog.
pub fn catalog() -> &'static [InvariantSpec] {
    INVARIANT_CATALOG
}

/// Looks up an invariant by stable identifier.
pub fn invariant_by_id(id: &str) -> Option<&'static InvariantSpec> {
    catalog().iter().find(|entry| entry.id == id)
}

/// Validates catalog shape constraints and identifier ordering.
pub fn validate_catalog(entries: &[InvariantSpec]) -> Result<(), InvariantCatalogError> {
    if entries.is_empty() {
        return Err(InvariantCatalogError::EmptyCatalog);
    }

    let mut previous_id: Option<&str> = None;
    for entry in entries {
        if entry.id.trim().is_empty() {
            return Err(InvariantCatalogError::EmptyInvariantId);
        }
        if entry.failure_codes.is_empty() {
            return Err(InvariantCatalogError::MissingFailureCodes(
                entry.id.to_owned(),
            ));
        }
        if let Some(previous) = previous_id {
            if entry.id == previous {
                return Err(InvariantCatalogError::DuplicateId(entry.id.to_owned()));
            }
            if entry.id < previous {
                return Err(InvariantCatalogError::Unsorted {
                    previous: previous.to_owned(),
                    current: entry.id.to_owned(),
                });
            }
        }
        previous_id = Some(entry.id);
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Invariant violation instance produced by taxonomy classification.
pub struct InvariantViolation {
    /// Stable invariant identifier.
    pub invariant_id: &'static str,
    /// Failure code for this violation.
    pub failure_code: InvariantFailureCode,
    /// Human-readable violation message.
    pub message: String,
}

/// Maps a transaction guard error to a deterministic invariant violation.
pub fn classify_transaction_guard_error(error: &TransactionGuardError) -> InvariantViolation {
    match error {
        TransactionGuardError::EmptyField(_) => InvariantViolation {
            invariant_id: "INV-TX-001",
            failure_code: InvariantFailureCode::EmptyField,
            message: error.to_string(),
        },
        TransactionGuardError::InvalidNonce(_) => InvariantViolation {
            invariant_id: "INV-TX-002",
            failure_code: InvariantFailureCode::InvalidNonce,
            message: error.to_string(),
        },
        TransactionGuardError::NonceOutOfSequence { .. } => InvariantViolation {
            invariant_id: "INV-TX-002",
            failure_code: InvariantFailureCode::NonceOutOfSequence,
            message: error.to_string(),
        },
        TransactionGuardError::InvalidSignature { .. } => InvariantViolation {
            invariant_id: "INV-TX-003",
            failure_code: InvariantFailureCode::InvalidSignature,
            message: error.to_string(),
        },
        TransactionGuardError::StateHashMismatch { .. } => InvariantViolation {
            invariant_id: "INV-TX-004",
            failure_code: InvariantFailureCode::StateHashMismatch,
            message: error.to_string(),
        },
        TransactionGuardError::DuplicateTransactionId(_) => InvariantViolation {
            invariant_id: "INV-TX-005",
            failure_code: InvariantFailureCode::DuplicateTransactionId,
            message: error.to_string(),
        },
        TransactionGuardError::UnvalidatedCommittedTransaction(_) => InvariantViolation {
            invariant_id: "INV-TX-006",
            failure_code: InvariantFailureCode::UnvalidatedCommittedTransaction,
            message: error.to_string(),
        },
    }
}

/// Maps a smoke-test error to an optional invariant violation.
pub fn classify_smoke_error(error: &SmokeError) -> Option<InvariantViolation> {
    match error {
        SmokeError::Guard(guard_error) => Some(classify_transaction_guard_error(guard_error)),
        SmokeError::EmptyMempool(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error taxonomy for invariant catalog validation failures.
pub enum InvariantCatalogError {
    /// Duplicate invariant identifier encountered.
    DuplicateId(String),
    /// Catalog is empty.
    EmptyCatalog,
    /// Invariant identifier is empty.
    EmptyInvariantId,
    /// Invariant has no failure-code mappings.
    MissingFailureCodes(String),
    /// Catalog ordering is not lexicographically sorted by identifier.
    Unsorted {
        /// Previous identifier in sequence.
        previous: String,
        /// Current identifier that violated ordering.
        current: String,
    },
}

impl fmt::Display for InvariantCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate invariant id: {id}"),
            Self::EmptyCatalog => write!(f, "invariant catalog must not be empty"),
            Self::EmptyInvariantId => write!(f, "invariant id must not be empty"),
            Self::MissingFailureCodes(id) => {
                write!(f, "invariant must define failure codes: {id}")
            }
            Self::Unsorted { previous, current } => {
                write!(
                    f,
                    "invariant catalog must be sorted by id; found {current} after {previous}"
                )
            }
        }
    }
}

impl std::error::Error for InvariantCatalogError {}

#[cfg(test)]
mod tests {
    use super::{
        catalog, classify_transaction_guard_error, validate_catalog, InvariantCatalogError,
        InvariantFailureCode, InvariantSpec,
    };
    use crate::transaction::TransactionGuardError;

    #[test]
    fn default_catalog_is_valid() {
        assert!(validate_catalog(catalog()).is_ok());
    }

    #[test]
    fn catalog_validation_rejects_duplicates() {
        let entry = InvariantSpec {
            id: "INV-TX-001",
            domain: super::InvariantDomain::Transactions,
            description: "duplicate",
            prd_refs: &["2.2.3"],
            owner_issue: "#78",
            failure_codes: &[InvariantFailureCode::EmptyField],
        };

        let entries = [entry.clone(), entry];
        assert_eq!(
            validate_catalog(&entries),
            Err(InvariantCatalogError::DuplicateId("INV-TX-001".to_owned()))
        );
    }

    #[test]
    fn taxonomy_maps_guard_errors() {
        let violation = classify_transaction_guard_error(&TransactionGuardError::InvalidNonce(0));
        assert_eq!(violation.invariant_id, "INV-TX-002");
        assert_eq!(violation.failure_code, InvariantFailureCode::InvalidNonce);
        assert!(violation.message.contains("nonce"));
    }

    #[test]
    fn taxonomy_maps_state_hash_to_stable_id() {
        // Regression: #77
        let violation =
            classify_transaction_guard_error(&TransactionGuardError::StateHashMismatch {
                expected: "state:1".to_owned(),
                found: "state:0".to_owned(),
            });

        assert_eq!(violation.invariant_id, "INV-TX-004");
        assert_eq!(
            violation.failure_code,
            InvariantFailureCode::StateHashMismatch
        );
    }
}
