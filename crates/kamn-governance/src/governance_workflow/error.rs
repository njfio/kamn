use super::models::GovernanceProposalStatus;
use std::fmt;

/// Errors emitted by governance proposal, vote, and execution flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovernanceWorkflowError {
    /// Required string field is empty.
    EmptyField(&'static str),
    /// DID failed canonical parsing/validation.
    InvalidDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Timestamp field must be positive.
    InvalidTimestamp(&'static str),
    /// Voting deadline does not occur after creation timestamp.
    InvalidDeadline {
        created_at_unix: u64,
        voting_deadline_unix: u64,
    },
    /// Quorum threshold must be positive.
    InvalidQuorum(usize),
    /// Target version could not be parsed as semantic version.
    InvalidParameterTargetVersion(String),
    /// Parameter min/max bounds are internally inconsistent.
    InvalidParameterRange {
        key: String,
        min_value: u64,
        max_value: u64,
    },
    /// Parameter key is not recognized by policy catalog.
    UnknownParameterKey(String),
    /// Requested parameter range exceeds policy-approved range.
    ParameterRangeOutsidePolicy {
        key: String,
        min_value: u64,
        max_value: u64,
        policy_min_value: u64,
        policy_max_value: u64,
    },
    /// Parameter is unsupported for the requested runtime version.
    ParameterUnsupportedForVersion {
        key: String,
        target_version: String,
        min_supported_version: String,
    },
    /// Proposed value lies outside requested min/max bounds.
    ParameterOutOfBounds {
        key: String,
        proposed_value: u64,
        min_value: u64,
        max_value: u64,
    },
    /// Proposal identifier already exists.
    DuplicateProposal(String),
    /// Proposal identifier does not exist.
    ProposalNotFound(String),
    /// Voter already cast a vote for this proposal.
    DuplicateVote {
        proposal_id: String,
        voter_did: String,
    },
    /// Proposal is no longer in voting state.
    ProposalClosed {
        proposal_id: String,
        status: GovernanceProposalStatus,
    },
    /// Proposal is not approved and cannot be executed.
    ProposalNotApproved {
        proposal_id: String,
        status: GovernanceProposalStatus,
    },
    /// Proposal already executed.
    AlreadyExecuted(String),
}

impl fmt::Display for GovernanceWorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid { field, reason_code, detail } => {
                write!(f, "invalid did field {field}: {reason_code} ({detail})")
            }
            Self::InvalidTimestamp(field) => write!(f, "timestamp must be > 0: {field}"),
            Self::InvalidDeadline { created_at_unix, voting_deadline_unix } => write!(
                f,
                "invalid voting deadline: created_at_unix={created_at_unix}, voting_deadline_unix={voting_deadline_unix}"
            ),
            Self::InvalidQuorum(value) => write!(f, "invalid quorum threshold: {value}"),
            Self::InvalidParameterTargetVersion(value) => {
                write!(f, "invalid parameter target version: {value}")
            }
            Self::InvalidParameterRange { key, min_value, max_value } => write!(
                f,
                "invalid parameter range: key={key}, min_value={min_value}, max_value={max_value}"
            ),
            Self::UnknownParameterKey(key) => write!(f, "unknown governance parameter key: {key}"),
            Self::ParameterRangeOutsidePolicy {
                key,
                min_value,
                max_value,
                policy_min_value,
                policy_max_value,
            } => write!(
                f,
                "parameter range outside policy: key={key}, min_value={min_value}, max_value={max_value}, policy_min_value={policy_min_value}, policy_max_value={policy_max_value}"
            ),
            Self::ParameterUnsupportedForVersion { key, target_version, min_supported_version } => {
                write!(
                    f,
                    "parameter key unsupported for target version: key={key}, target_version={target_version}, min_supported_version={min_supported_version}"
                )
            }
            Self::ParameterOutOfBounds { key, proposed_value, min_value, max_value } => write!(
                f,
                "parameter value out of bounds: key={key}, proposed_value={proposed_value}, min_value={min_value}, max_value={max_value}"
            ),
            Self::DuplicateProposal(proposal_id) => {
                write!(f, "duplicate governance proposal id: {proposal_id}")
            }
            Self::ProposalNotFound(proposal_id) => {
                write!(f, "governance proposal not found: {proposal_id}")
            }
            Self::DuplicateVote { proposal_id, voter_did } => write!(
                f,
                "duplicate governance vote: proposal={proposal_id}, voter={voter_did}"
            ),
            Self::ProposalClosed { proposal_id, status } => write!(
                f,
                "proposal is closed for voting: proposal={proposal_id}, status={status:?}"
            ),
            Self::ProposalNotApproved { proposal_id, status } => write!(
                f,
                "proposal is not approved for execution: proposal={proposal_id}, status={status:?}"
            ),
            Self::AlreadyExecuted(proposal_id) => write!(f, "proposal already executed: {proposal_id}"),
        }
    }
}

impl std::error::Error for GovernanceWorkflowError {}
