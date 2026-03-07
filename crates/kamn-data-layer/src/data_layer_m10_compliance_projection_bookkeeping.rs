//! Deterministic M10 shred-completeness projection bookkeeping behind the
//! core-agnostic compliance projection port seam.

use std::{collections::BTreeSet, fmt};

use crate::{
    data_layer_m10_validate_partition_month_id, DataLayerM10ComplianceProjectionPort,
    DataLayerM10ComplianceProjectionPortError, DataLayerM10PartitionRegistryStateMachine,
    DataLayerM10PartitionRegistryStateMachineError,
};

/// Stable reason marker when M10 partition shred-completeness projection is applied.
pub const DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE: &str =
    "m10_partition_compliance_projection_applied";
/// Stable reason marker when M8 projection resolves that partition is not fully shredded.
pub const DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE: &str =
    "m10_partition_compliance_shred_incomplete";
/// Stable reason marker when M8 projection resolves that partition is fully shredded.
pub const DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE: &str =
    "m10_partition_compliance_shred_complete";
/// Stable reason marker when M8 projection resolves that legal hold still blocks archival.
pub const DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE: &str =
    "m10_partition_compliance_legal_hold_active";
/// Stable reason marker for owner-scope projection denials.
pub const DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE: &str =
    "m10_partition_compliance_owner_scope_denied";
/// Stable reason marker when M8 lookup fails during projection.
pub const DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE: &str =
    "m10_partition_compliance_lookup_failed";
/// Stable reason marker when M8 projection input is invalid.
pub const DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE: &str =
    "m10_partition_compliance_input_invalid";

/// Projection request to derive partition shred completeness from a compliance projection port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ComplianceShredProjectionRequest {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Message identifiers that belong to the partition scope.
    pub partition_message_ids: Vec<String>,
}

/// Projection report for port-derived partition shred completeness.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM10ComplianceShredProjectionReport {
    /// Partition month identifier as `YYYYMM`.
    pub partition_month_id: u32,
    /// Canonical partition name `messages_YYYY_MM`.
    pub partition_name: String,
    /// Total message identifiers evaluated from projection input.
    pub total_partition_messages: usize,
    /// Number of messages currently shredded in the compliance source.
    pub shredded_partition_messages: usize,
    /// Derived partition completeness marker.
    pub all_messages_shredded: bool,
    /// Completeness reason marker (`complete`, `incomplete`, or `legal hold`).
    pub reason_code: &'static str,
    /// Stable projection-applied marker.
    pub projection_reason_code: &'static str,
}

/// Error taxonomy for extracted M10 compliance-projection bookkeeping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM10ComplianceProjectionBookkeepingError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Partition month id failed `YYYYMM` validation.
    InvalidPartitionMonthId(u32),
    /// Owner-scope authorization denied.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Projection lookup failed.
    PortLookupFailed {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail string.
        detail: String,
    },
    /// Projection input was invalid.
    PortInvalidInput {
        /// Stable reason marker.
        reason_code: &'static str,
        /// Stable detail string.
        detail: String,
    },
    /// Target partition state was missing.
    PartitionNotFound(String),
    /// Registry mutation failed in an unexpected way.
    RegistryMutationFailed(String),
}

impl fmt::Display for DataLayerM10ComplianceProjectionBookkeepingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "field must not be empty: {field}"),
            Self::InvalidPartitionMonthId(value) => {
                write!(formatter, "invalid partition month id: {value}")
            }
            Self::OwnerScopeViolation { reason_code } => {
                write!(formatter, "owner scope violation: {reason_code}")
            }
            Self::PortLookupFailed {
                reason_code,
                detail,
            } => write!(formatter, "lookup failed: {reason_code} ({detail})"),
            Self::PortInvalidInput {
                reason_code,
                detail,
            } => write!(formatter, "invalid input: {reason_code} ({detail})"),
            Self::PartitionNotFound(name) => write!(formatter, "partition not found: {name}"),
            Self::RegistryMutationFailed(detail) => {
                write!(formatter, "registry mutation failed: {detail}")
            }
        }
    }
}

impl std::error::Error for DataLayerM10ComplianceProjectionBookkeepingError {}

/// Projects partition shred completeness through the extracted port and state-machine seams.
pub fn data_layer_m10_project_partition_shred_completeness_with_port(
    state_machine: &mut DataLayerM10PartitionRegistryStateMachine,
    compliance_port: &impl DataLayerM10ComplianceProjectionPort,
    request: DataLayerM10ComplianceShredProjectionRequest,
) -> Result<
    DataLayerM10ComplianceShredProjectionReport,
    DataLayerM10ComplianceProjectionBookkeepingError,
> {
    let owner_did = compliance_port
        .authorize_owner_scope(
            request.requester_owner_did.as_str(),
            request.owner_did.as_str(),
        )
        .map_err(map_projection_port_error)?;
    validate_partition_month_id(request.partition_month_id)?;
    let message_ids = collect_partition_message_ids(request.partition_message_ids)?;
    let total_partition_messages = message_ids.len();
    let (shredded_partition_messages, legal_hold_active_messages) =
        evaluate_partition_message_shred_completeness(
            compliance_port,
            owner_did.as_str(),
            &message_ids,
        )?;
    let all_messages_shredded = shredded_partition_messages == total_partition_messages;
    let reason_code = if legal_hold_active_messages > 0 {
        DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
    } else if all_messages_shredded {
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
    } else {
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
    };
    let record = state_machine
        .apply_partition_shred_completeness(
            request.partition_month_id,
            all_messages_shredded,
            DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
        )
        .map_err(map_state_machine_error)?;

    Ok(DataLayerM10ComplianceShredProjectionReport {
        partition_month_id: record.partition_month_id,
        partition_name: record.partition_name,
        total_partition_messages,
        shredded_partition_messages,
        all_messages_shredded,
        reason_code,
        projection_reason_code: DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE,
    })
}

fn validate_partition_month_id(
    partition_month_id: u32,
) -> Result<(), DataLayerM10ComplianceProjectionBookkeepingError> {
    data_layer_m10_validate_partition_month_id(partition_month_id)
        .map_err(|_| DataLayerM10ComplianceProjectionBookkeepingError::InvalidPartitionMonthId(partition_month_id))
}

fn collect_partition_message_ids(
    partition_message_ids: Vec<String>,
) -> Result<BTreeSet<String>, DataLayerM10ComplianceProjectionBookkeepingError> {
    if partition_message_ids.is_empty() {
        return Err(DataLayerM10ComplianceProjectionBookkeepingError::EmptyField(
            "partition_message_ids",
        ));
    }
    let mut message_ids = BTreeSet::new();
    for message_id in partition_message_ids {
        if message_id.trim().is_empty() {
            return Err(DataLayerM10ComplianceProjectionBookkeepingError::EmptyField(
                "partition_message_ids",
            ));
        }
        message_ids.insert(message_id);
    }
    Ok(message_ids)
}

fn evaluate_partition_message_shred_completeness(
    compliance_port: &impl DataLayerM10ComplianceProjectionPort,
    owner_did: &str,
    message_ids: &BTreeSet<String>,
) -> Result<(usize, usize), DataLayerM10ComplianceProjectionBookkeepingError> {
    let mut shredded_partition_messages = 0usize;
    let mut legal_hold_active_messages = 0usize;
    for message_id in message_ids {
        let message = compliance_port
            .message_for_owner(owner_did, message_id.as_str())
            .map_err(map_projection_port_error)?;
        if message.legal_hold_active {
            legal_hold_active_messages += 1;
        }
        if message.shredded_at_epoch_seconds.is_some() {
            shredded_partition_messages += 1;
        }
    }
    Ok((shredded_partition_messages, legal_hold_active_messages))
}

fn map_projection_port_error(
    error: DataLayerM10ComplianceProjectionPortError,
) -> DataLayerM10ComplianceProjectionBookkeepingError {
    match error {
        DataLayerM10ComplianceProjectionPortError::OwnerScopeViolation => {
            DataLayerM10ComplianceProjectionBookkeepingError::OwnerScopeViolation {
                reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
            }
        }
        DataLayerM10ComplianceProjectionPortError::LookupFailed(detail) => {
            DataLayerM10ComplianceProjectionBookkeepingError::PortLookupFailed {
                reason_code: DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
                detail,
            }
        }
        DataLayerM10ComplianceProjectionPortError::InvalidInput(detail) => {
            DataLayerM10ComplianceProjectionBookkeepingError::PortInvalidInput {
                reason_code: DATA_LAYER_M10_COMPLIANCE_INPUT_INVALID_REASON_CODE,
                detail,
            }
        }
    }
}

fn map_state_machine_error(
    error: DataLayerM10PartitionRegistryStateMachineError,
) -> DataLayerM10ComplianceProjectionBookkeepingError {
    match error {
        DataLayerM10PartitionRegistryStateMachineError::EmptyField(field) => {
            DataLayerM10ComplianceProjectionBookkeepingError::EmptyField(field)
        }
        DataLayerM10PartitionRegistryStateMachineError::InvalidPartitionMonthId(value) => {
            DataLayerM10ComplianceProjectionBookkeepingError::InvalidPartitionMonthId(value)
        }
        DataLayerM10PartitionRegistryStateMachineError::PartitionNotFound(name) => {
            DataLayerM10ComplianceProjectionBookkeepingError::PartitionNotFound(name)
        }
        other => DataLayerM10ComplianceProjectionBookkeepingError::RegistryMutationFailed(
            other.to_string(),
        ),
    }
}
