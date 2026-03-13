use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedDidHandshakeInput {
    pub handshake_id: String,
    pub subject_did: String,
    pub local_network: String,
    pub remote_network: String,
    pub resolver_version: String,
    pub signature_policy_passed: bool,
    pub nonce_monotonic: bool,
    pub downgrade_detected: bool,
    pub partition_sequence_monotonic: bool,
    pub required_quorum: u16,
    pub received_quorum: u16,
}

impl FederatedDidHandshakeInput {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        handshake_id: &str, subject_did: &str, local_network: &str, remote_network: &str,
        resolver_version: &str, signature_policy_passed: bool, nonce_monotonic: bool,
        downgrade_detected: bool, partition_sequence_monotonic: bool, required_quorum: u16,
        received_quorum: u16,
    ) -> Result<Self, FederatedDidHandshakeError> {
        validate_required_inputs(handshake_id, subject_did, local_network, remote_network, required_quorum)?;
        Ok(build_input(
            handshake_id, subject_did, local_network, remote_network, resolver_version,
            signature_policy_passed, nonce_monotonic, downgrade_detected,
            partition_sequence_monotonic, required_quorum, received_quorum,
        ))
    }
}

fn build_input(
    handshake_id: &str, subject_did: &str, local_network: &str, remote_network: &str,
    resolver_version: &str, signature_policy_passed: bool, nonce_monotonic: bool,
    downgrade_detected: bool, partition_sequence_monotonic: bool, required_quorum: u16,
    received_quorum: u16,
) -> FederatedDidHandshakeInput {
    FederatedDidHandshakeInput {
        handshake_id: handshake_id.trim().to_owned(),
        subject_did: subject_did.trim().to_owned(),
        local_network: local_network.trim().to_owned(),
        remote_network: remote_network.trim().to_owned(),
        resolver_version: resolver_version.trim().to_owned(),
        signature_policy_passed,
        nonce_monotonic,
        downgrade_detected,
        partition_sequence_monotonic,
        required_quorum,
        received_quorum,
    }
}

fn validate_required_inputs(
    handshake_id: &str,
    subject_did: &str,
    local_network: &str,
    remote_network: &str,
    required_quorum: u16,
) -> Result<(), FederatedDidHandshakeError> {
    for (field, value) in [("handshake_id", handshake_id), ("subject_did", subject_did), ("local_network", local_network), ("remote_network", remote_network)] {
        if value.trim().is_empty() {
            return Err(FederatedDidHandshakeError::EmptyField(field));
        }
    }
    if required_quorum == 0 {
        return Err(FederatedDidHandshakeError::InvalidRequiredQuorum { required: required_quorum });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FederatedDidHandshakeDecision {
    pub handshake_id: String,
    pub subject_did: String,
    pub local_network: String,
    pub remote_network: String,
}

impl FederatedDidHandshakeDecision {
    pub fn reason_code(&self) -> &'static str {
        "federated_did_handshake_ok"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FederatedDidHandshakeError {
    EmptyField(&'static str),
    InvalidRequiredQuorum { required: u16 },
    ResolverVersionMissing { handshake_id: String },
    TrustStoreMiss { subject_did: String, network: String },
    SignaturePolicyFailed { handshake_id: String },
    QuorumShortfall { required: u16, received: u16 },
    NonceReplayDetected { handshake_id: String },
    PartitionSequenceReplayDetected { handshake_id: String },
    DowngradeDetected { handshake_id: String },
}

impl FederatedDidHandshakeError {
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::EmptyField(_) => "federated_did_handshake_invalid_input",
            Self::InvalidRequiredQuorum { .. } => "federated_did_handshake_invalid_quorum",
            Self::ResolverVersionMissing { .. } => "federated_did_handshake_resolver_missing",
            Self::TrustStoreMiss { .. } => "federated_did_handshake_trust_store_miss",
            Self::SignaturePolicyFailed { .. } => "federated_did_handshake_signature_policy_failed",
            Self::QuorumShortfall { .. } => "federated_did_handshake_quorum_shortfall",
            Self::NonceReplayDetected { .. } => "federated_did_handshake_nonce_replay",
            Self::PartitionSequenceReplayDetected { .. } => "federated_did_handshake_partition_replay",
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
        FederatedDidHandshakeError::EmptyField(field) => format!("federated did handshake field is empty: {field}"),
        FederatedDidHandshakeError::InvalidRequiredQuorum { required } => format!("invalid required quorum for federated did handshake: {required}"),
        FederatedDidHandshakeError::ResolverVersionMissing { handshake_id } => format!("resolver version missing for federated did handshake: {handshake_id}"),
        FederatedDidHandshakeError::TrustStoreMiss { subject_did, network } => format!("federated did handshake trust-store miss for did {subject_did} on network {network}"),
        FederatedDidHandshakeError::SignaturePolicyFailed { handshake_id } => format!("federated did handshake signature policy failed: {handshake_id}"),
        FederatedDidHandshakeError::QuorumShortfall { required, received } => format!("federated did handshake quorum shortfall: required {required}, received {received}"),
        FederatedDidHandshakeError::NonceReplayDetected { handshake_id } => format!("federated did handshake nonce replay detected: {handshake_id}"),
        FederatedDidHandshakeError::PartitionSequenceReplayDetected { handshake_id } => format!("federated did handshake partition sequence replay detected: {handshake_id}"),
        FederatedDidHandshakeError::DowngradeDetected { handshake_id } => format!("federated did handshake downgrade detected: {handshake_id}"),
    }
}
