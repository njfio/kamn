use super::{
    FederatedDidHandshakeDecision, FederatedDidHandshakeError, FederatedDidHandshakeInput,
    FederatedDidTrustStore,
};

#[derive(Debug, Clone)]
pub struct FederatedDidHandshakeEvaluator<T: FederatedDidTrustStore> {
    trust_store: T,
}

impl<T: FederatedDidTrustStore> FederatedDidHandshakeEvaluator<T> {
    pub fn new(trust_store: T) -> Self {
        Self { trust_store }
    }

    pub fn evaluate(
        &mut self,
        input: FederatedDidHandshakeInput,
    ) -> Result<FederatedDidHandshakeDecision, FederatedDidHandshakeError> {
        ensure_trusted(&self.trust_store, &input)?;
        ensure_resolver_and_signature(&input)?;
        ensure_quorum(&input)?;
        validate_runtime_guards(&input)?;
        Ok(FederatedDidHandshakeDecision {
            handshake_id: input.handshake_id,
            subject_did: input.subject_did,
            local_network: input.local_network,
            remote_network: input.remote_network,
        })
    }
}

fn ensure_trusted<T: FederatedDidTrustStore>(
    trust_store: &T,
    input: &FederatedDidHandshakeInput,
) -> Result<(), FederatedDidHandshakeError> {
    if trust_store.is_trusted(&input.remote_network, &input.subject_did) {
        return Ok(());
    }
    Err(FederatedDidHandshakeError::TrustStoreMiss {
        subject_did: input.subject_did.clone(),
        network: input.remote_network.clone(),
    })
}

fn ensure_resolver_and_signature(
    input: &FederatedDidHandshakeInput,
) -> Result<(), FederatedDidHandshakeError> {
    if input.resolver_version.trim().is_empty() {
        return Err(FederatedDidHandshakeError::ResolverVersionMissing { handshake_id: input.handshake_id.clone() });
    }
    if input.signature_policy_passed {
        return Ok(());
    }
    Err(FederatedDidHandshakeError::SignaturePolicyFailed { handshake_id: input.handshake_id.clone() })
}

fn ensure_quorum(input: &FederatedDidHandshakeInput) -> Result<(), FederatedDidHandshakeError> {
    if input.received_quorum >= input.required_quorum {
        return Ok(());
    }
    Err(FederatedDidHandshakeError::QuorumShortfall {
        required: input.required_quorum,
        received: input.received_quorum,
    })
}

fn validate_runtime_guards(
    input: &FederatedDidHandshakeInput,
) -> Result<(), FederatedDidHandshakeError> {
    if !input.nonce_monotonic {
        return Err(FederatedDidHandshakeError::NonceReplayDetected { handshake_id: input.handshake_id.clone() });
    }
    if !input.partition_sequence_monotonic {
        return Err(FederatedDidHandshakeError::PartitionSequenceReplayDetected { handshake_id: input.handshake_id.clone() });
    }
    if input.downgrade_detected {
        return Err(FederatedDidHandshakeError::DowngradeDetected { handshake_id: input.handshake_id.clone() });
    }
    Ok(())
}
