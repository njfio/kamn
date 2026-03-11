use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusEvaluator {
    required_quorum: usize,
    consumed_attestation_ids: BTreeSet<String>,
}

impl ValidatorProofConsensusEvaluator {
    pub fn new(required_quorum: usize) -> Result<Self, ValidatorProofConsensusError> {
        if required_quorum == 0 {
            return Err(ValidatorProofConsensusError::InvalidRequiredQuorum(
                required_quorum,
            ));
        }
        Ok(Self {
            required_quorum,
            consumed_attestation_ids: BTreeSet::new(),
        })
    }

    pub fn required_quorum(&self) -> usize {
        self.required_quorum
    }

    pub fn evaluate(
        &mut self,
        input: ValidatorProofConsensusInput,
    ) -> Result<ValidatorProofConsensusDecision, ValidatorProofConsensusError> {
        ensure_quorum(input.attestations.len(), self.required_quorum)?;
        let mut state = EvaluationState::default();
        for attestation in &input.attestations {
            validate_attestation(
                attestation,
                &input,
                &self.consumed_attestation_ids,
                &mut state,
            )?;
        }
        let local_attestation_ids = std::mem::take(&mut state.local_attestation_ids);
        consume_attestations(&mut self.consumed_attestation_ids, local_attestation_ids);
        Ok(build_decision(input, self.required_quorum, state))
    }
}

#[derive(Default)]
struct EvaluationState {
    validator_dids: BTreeSet<String>,
    local_attestation_ids: BTreeSet<String>,
    valid_attestation_count: usize,
    invalid_attestation_count: usize,
    replay_attestation_count: usize,
}

fn ensure_quorum(
    received: usize,
    required_quorum: usize,
) -> Result<(), ValidatorProofConsensusError> {
    if received >= required_quorum {
        return Ok(());
    }
    Err(ValidatorProofConsensusError::InsufficientAttestations {
        required: required_quorum,
        received,
    })
}

fn validate_attestation(
    attestation: &ValidatorProofAttestation,
    input: &ValidatorProofConsensusInput,
    consumed_attestation_ids: &BTreeSet<String>,
    state: &mut EvaluationState,
) -> Result<(), ValidatorProofConsensusError> {
    validate_attestation_identity(attestation, input)?;
    validate_validator_uniqueness(attestation, &mut state.validator_dids)?;
    validate_attestation_id(
        attestation,
        consumed_attestation_ids,
        &mut state.local_attestation_ids,
    )?;
    count_verdict(attestation.verdict, state);
    Ok(())
}

fn validate_attestation_identity(
    attestation: &ValidatorProofAttestation,
    input: &ValidatorProofConsensusInput,
) -> Result<(), ValidatorProofConsensusError> {
    if attestation.message_id != input.message_id {
        return Err(ValidatorProofConsensusError::AttestationMessageMismatch {
            expected: input.message_id.clone(),
            found: attestation.message_id.clone(),
        });
    }
    if attestation.artifact_id != input.artifact_id {
        return Err(ValidatorProofConsensusError::AttestationArtifactMismatch {
            expected: input.artifact_id.clone(),
            found: attestation.artifact_id.clone(),
        });
    }
    Ok(())
}

fn validate_validator_uniqueness(
    attestation: &ValidatorProofAttestation,
    validator_dids: &mut BTreeSet<String>,
) -> Result<(), ValidatorProofConsensusError> {
    if validator_dids.insert(attestation.validator_did.clone()) {
        return Ok(());
    }
    Err(ValidatorProofConsensusError::DuplicateValidator(
        attestation.validator_did.clone(),
    ))
}

fn validate_attestation_id(
    attestation: &ValidatorProofAttestation,
    consumed_attestation_ids: &BTreeSet<String>,
    local_attestation_ids: &mut BTreeSet<String>,
) -> Result<(), ValidatorProofConsensusError> {
    if consumed_attestation_ids.contains(&attestation.attestation_id) {
        return Err(ValidatorProofConsensusError::AttestationReplay(
            attestation.attestation_id.clone(),
        ));
    }
    if local_attestation_ids.insert(attestation.attestation_id.clone()) {
        return Ok(());
    }
    Err(ValidatorProofConsensusError::DuplicateAttestationId(
        attestation.attestation_id.clone(),
    ))
}

fn count_verdict(verdict: ValidatorProofVerdict, state: &mut EvaluationState) {
    match verdict {
        ValidatorProofVerdict::Valid => state.valid_attestation_count += 1,
        ValidatorProofVerdict::Invalid => state.invalid_attestation_count += 1,
        ValidatorProofVerdict::Replay => state.replay_attestation_count += 1,
    }
}

fn consume_attestations(
    consumed_attestation_ids: &mut BTreeSet<String>,
    local_attestation_ids: BTreeSet<String>,
) {
    for attestation_id in local_attestation_ids {
        consumed_attestation_ids.insert(attestation_id);
    }
}

fn build_decision(
    input: ValidatorProofConsensusInput,
    required_quorum: usize,
    state: EvaluationState,
) -> ValidatorProofConsensusDecision {
    ValidatorProofConsensusDecision {
        message_id: input.message_id,
        artifact_id: input.artifact_id,
        required_quorum,
        validator_count: state.validator_dids.len(),
        validator_dids: state.validator_dids.into_iter().collect(),
        valid_attestation_count: state.valid_attestation_count,
        invalid_attestation_count: state.invalid_attestation_count,
        replay_attestation_count: state.replay_attestation_count,
        status: consensus_status(
            state.valid_attestation_count,
            state.invalid_attestation_count,
            state.replay_attestation_count,
        ),
    }
}
