use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Federated Did Handshake Input.
pub struct FederatedDidHandshakeInput {
    /// Handshake id carried by this public contract model.
    pub handshake_id: String,
    /// Subject did carried by this public contract model.
    pub subject_did: String,
    /// Local network carried by this public contract model.
    pub local_network: String,
    /// Remote network carried by this public contract model.
    pub remote_network: String,
    /// Resolver version carried by this public contract model.
    pub resolver_version: String,
    /// Signature policy passed carried by this public contract model.
    pub signature_policy_passed: bool,
    /// Nonce monotonic carried by this public contract model.
    pub nonce_monotonic: bool,
    /// Downgrade detected carried by this public contract model.
    pub downgrade_detected: bool,
    /// Partition sequence monotonic carried by this public contract model.
    pub partition_sequence_monotonic: bool,
    /// Required quorum carried by this public contract model.
    pub required_quorum: u16,
    /// Received quorum carried by this public contract model.
    pub received_quorum: u16,
}

struct FederatedDidHandshakeParts<'a> {
    handshake_id: &'a str,
    subject_did: &'a str,
    local_network: &'a str,
    remote_network: &'a str,
    resolver_version: &'a str,
    signature_policy_passed: bool,
    nonce_monotonic: bool,
    downgrade_detected: bool,
    partition_sequence_monotonic: bool,
    required_quorum: u16,
    received_quorum: u16,
}

impl FederatedDidHandshakeInput {
    #[allow(clippy::too_many_arguments)]
    /// Creates a new value for this public contract type.
    pub fn new(
        handshake_id: &str,
        subject_did: &str,
        local_network: &str,
        remote_network: &str,
        resolver_version: &str,
        signature_policy_passed: bool,
        nonce_monotonic: bool,
        downgrade_detected: bool,
        partition_sequence_monotonic: bool,
        required_quorum: u16,
        received_quorum: u16,
    ) -> Result<Self, FederatedDidHandshakeError> {
        validate_required_inputs(
            handshake_id,
            subject_did,
            local_network,
            remote_network,
            required_quorum,
        )?;
        Ok(build_input(FederatedDidHandshakeParts {
            handshake_id,
            subject_did,
            local_network,
            remote_network,
            resolver_version,
            signature_policy_passed,
            nonce_monotonic,
            downgrade_detected,
            partition_sequence_monotonic,
            required_quorum,
            received_quorum,
        }))
    }
}

fn build_input(parts: FederatedDidHandshakeParts<'_>) -> FederatedDidHandshakeInput {
    FederatedDidHandshakeInput {
        handshake_id: parts.handshake_id.trim().to_owned(),
        subject_did: parts.subject_did.trim().to_owned(),
        local_network: parts.local_network.trim().to_owned(),
        remote_network: parts.remote_network.trim().to_owned(),
        resolver_version: parts.resolver_version.trim().to_owned(),
        signature_policy_passed: parts.signature_policy_passed,
        nonce_monotonic: parts.nonce_monotonic,
        downgrade_detected: parts.downgrade_detected,
        partition_sequence_monotonic: parts.partition_sequence_monotonic,
        required_quorum: parts.required_quorum,
        received_quorum: parts.received_quorum,
    }
}

fn validate_required_inputs(
    handshake_id: &str,
    subject_did: &str,
    local_network: &str,
    remote_network: &str,
    required_quorum: u16,
) -> Result<(), FederatedDidHandshakeError> {
    for (field, value) in [
        ("handshake_id", handshake_id),
        ("subject_did", subject_did),
        ("local_network", local_network),
        ("remote_network", remote_network),
    ] {
        if value.trim().is_empty() {
            return Err(FederatedDidHandshakeError::EmptyField(field));
        }
    }
    if required_quorum == 0 {
        return Err(FederatedDidHandshakeError::InvalidRequiredQuorum {
            required: required_quorum,
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract model for Federated Did Handshake Decision.
pub struct FederatedDidHandshakeDecision {
    /// Handshake id carried by this public contract model.
    pub handshake_id: String,
    /// Subject did carried by this public contract model.
    pub subject_did: String,
    /// Local network carried by this public contract model.
    pub local_network: String,
    /// Remote network carried by this public contract model.
    pub remote_network: String,
}

impl FederatedDidHandshakeDecision {
    /// Runs the reason code contract operation.
    pub fn reason_code(&self) -> &'static str {
        "federated_did_handshake_ok"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Public contract enum for Federated Did Handshake Error.
pub enum FederatedDidHandshakeError {
    /// Empty field variant for this public contract enum.
    EmptyField(&'static str),
    /// Invalid required quorum variant for this public contract enum.
    InvalidRequiredQuorum {
        /// U16 carried by this public contract model.
        required: u16,
    },
    /// Resolver version missing variant for this public contract enum.
    ResolverVersionMissing {
        /// String carried by this public contract model.
        handshake_id: String,
    },
    /// Trust store miss variant for this public contract enum.
    TrustStoreMiss {
        /// String carried by this public contract model.
        subject_did: String,
        /// String carried by this public contract model.
        network: String,
    },
    /// Signature policy failed variant for this public contract enum.
    SignaturePolicyFailed {
        /// String carried by this public contract model.
        handshake_id: String,
    },
    /// Quorum shortfall variant for this public contract enum.
    QuorumShortfall {
        /// U16 carried by this public contract model.
        required: u16,
        /// U16 carried by this public contract model.
        received: u16,
    },
    /// Nonce replay detected variant for this public contract enum.
    NonceReplayDetected {
        /// String carried by this public contract model.
        handshake_id: String,
    },
    /// Partition sequence replay detected variant for this public contract enum.
    PartitionSequenceReplayDetected {
        /// String carried by this public contract model.
        handshake_id: String,
    },
    /// Downgrade detected variant for this public contract enum.
    DowngradeDetected {
        /// String carried by this public contract model.
        handshake_id: String,
    },
}

impl FederatedDidHandshakeError {
    /// Runs the reason code contract operation.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::EmptyField(_) => "federated_did_handshake_invalid_input",
            Self::InvalidRequiredQuorum { .. } => "federated_did_handshake_invalid_quorum",
            Self::ResolverVersionMissing { .. } => "federated_did_handshake_resolver_missing",
            Self::TrustStoreMiss { .. } => "federated_did_handshake_trust_store_miss",
            Self::SignaturePolicyFailed { .. } => "federated_did_handshake_signature_policy_failed",
            Self::QuorumShortfall { .. } => "federated_did_handshake_quorum_shortfall",
            Self::NonceReplayDetected { .. } => "federated_did_handshake_nonce_replay",
            Self::PartitionSequenceReplayDetected { .. } => {
                "federated_did_handshake_partition_replay"
            }
            Self::DowngradeDetected { .. } => "federated_did_handshake_downgrade_detected",
        }
    }
}

impl fmt::Display for FederatedDidHandshakeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", display_message(self))
    }
}

impl std::error::Error for FederatedDidHandshakeError {}

fn display_message(error: &FederatedDidHandshakeError) -> String {
    match error {
        FederatedDidHandshakeError::EmptyField(field) => {
            format!("federated did handshake field is empty: {field}")
        }
        FederatedDidHandshakeError::InvalidRequiredQuorum { required } => {
            format!("invalid required quorum for federated did handshake: {required}")
        }
        FederatedDidHandshakeError::ResolverVersionMissing { handshake_id } => {
            format!("resolver version missing for federated did handshake: {handshake_id}")
        }
        FederatedDidHandshakeError::TrustStoreMiss {
            subject_did,
            network,
        } => format!(
            "federated did handshake trust-store miss for did {subject_did} on network {network}"
        ),
        FederatedDidHandshakeError::SignaturePolicyFailed { handshake_id } => {
            format!("federated did handshake signature policy failed: {handshake_id}")
        }
        FederatedDidHandshakeError::QuorumShortfall { required, received } => format!(
            "federated did handshake quorum shortfall: required {required}, received {received}"
        ),
        FederatedDidHandshakeError::NonceReplayDetected { handshake_id } => {
            format!("federated did handshake nonce replay detected: {handshake_id}")
        }
        FederatedDidHandshakeError::PartitionSequenceReplayDetected { handshake_id } => {
            format!("federated did handshake partition sequence replay detected: {handshake_id}")
        }
        FederatedDidHandshakeError::DowngradeDetected { handshake_id } => {
            format!("federated did handshake downgrade detected: {handshake_id}")
        }
    }
}
