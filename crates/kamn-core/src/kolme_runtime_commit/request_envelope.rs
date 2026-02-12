//! Runtime commit request and signed-envelope ownership.

use super::{
    are_kolme_runtime_commit_request_fields_single_line_contract,
    deterministic_runtime_commit_idempotency_key_contract,
    is_kolme_canonical_runtime_commit_signed_message_contract,
    is_kolme_valid_runtime_nonce_input_contract,
    is_kolme_valid_runtime_operation_id_input_contract,
    is_kolme_valid_runtime_payload_hash_input_contract,
    is_kolme_valid_runtime_state_root_input_contract,
    is_kolme_valid_signed_envelope_message_input_contract,
    is_kolme_valid_signed_envelope_signature_input_contract,
    is_kolme_valid_signed_envelope_signer_key_id_input_contract,
    normalize_kolme_runtime_commit_request_fields_contract,
    normalize_kolme_runtime_commit_signed_envelope_fields_contract,
    render_kolme_runtime_commit_wire_payload_contract,
    render_kolme_signed_envelope_wire_payload_contract, AgentDid, KolmeApiBroadcastRequest,
    KolmeRuntimeCommitError,
};

/// Runtime commit submission request for the Kolme execution path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitRequest {
    /// Deterministic operation identifier.
    pub operation_id: String,
    /// Runtime state root/hash reference.
    pub state_root: String,
    /// Actor DID submitting the runtime commit.
    pub actor_did: AgentDid,
    /// Monotonic submission nonce.
    pub nonce: u64,
    /// Deterministic payload hash marker.
    pub payload_hash: String,
    idempotency_key: String,
}

impl KolmeRuntimeCommitRequest {
    /// Builds a deterministic commit request and validates required invariants.
    pub fn deterministic(
        operation_id: &str,
        state_root: &str,
        actor_did: &str,
        nonce: u64,
        payload_hash: &str,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let actor_did =
            AgentDid::parse(actor_did).map_err(|_| KolmeRuntimeCommitError::InvalidRequest {
                field: "actor_did",
                reason: "must be a valid KAMN DID",
            })?;
        let (operation_id, state_root, payload_hash) =
            normalize_kolme_runtime_commit_request_fields_contract(
                operation_id,
                state_root,
                payload_hash,
            );
        let actor_did_value = actor_did.as_str().to_owned();
        let idempotency_key = deterministic_runtime_commit_idempotency_key_contract(
            operation_id.as_str(),
            state_root.as_str(),
            actor_did_value.as_str(),
            nonce,
            payload_hash.as_str(),
        );

        let request = Self {
            operation_id,
            state_root,
            actor_did,
            nonce,
            payload_hash,
            idempotency_key,
        };
        request.validate()?;
        Ok(request)
    }

    /// Returns deterministic request payload in canonical field order.
    pub fn to_wire_payload(&self) -> String {
        render_kolme_runtime_commit_wire_payload_contract(
            self.operation_id.as_str(),
            self.state_root.as_str(),
            self.actor_did.as_str(),
            self.nonce,
            self.payload_hash.as_str(),
            self.idempotency_key.as_str(),
        )
    }

    /// Returns the deterministic idempotency key.
    pub fn idempotency_key(&self) -> &str {
        &self.idempotency_key
    }

    /// Translates a canonical runtime commit into a signed broadcast envelope.
    pub fn translate_to_signed_broadcast_envelope(
        &self,
        signer_key_id: &str,
        signed_message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<KolmeRuntimeCommitSignedBroadcastEnvelope, KolmeRuntimeCommitError> {
        self.validate()?;
        let canonical_message = self.to_wire_payload();
        if !is_kolme_canonical_runtime_commit_signed_message_contract(
            canonical_message.as_str(),
            signed_message,
        ) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signed_message",
                reason: "must match canonical runtime commit wire payload",
            });
        }
        KolmeRuntimeCommitSignedBroadcastEnvelope::new(
            signer_key_id,
            signed_message,
            signature,
            recovery_id,
        )
    }

    /// Validates commit request schema and invariant boundaries.
    pub fn validate(&self) -> Result<(), KolmeRuntimeCommitError> {
        if !is_kolme_valid_runtime_operation_id_input_contract(self.operation_id.as_str()) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "operation_id",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_runtime_state_root_input_contract(self.state_root.as_str()) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "state_root",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_runtime_nonce_input_contract(self.nonce) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "nonce",
                reason: "must be positive",
            });
        }
        if !is_kolme_valid_runtime_payload_hash_input_contract(self.payload_hash.as_str()) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "payload_hash",
                reason: "must not be empty",
            });
        }
        if !are_kolme_runtime_commit_request_fields_single_line_contract(
            self.operation_id.as_str(),
            self.state_root.as_str(),
            self.payload_hash.as_str(),
        ) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "wire_payload",
                reason: "fields must be single-line",
            });
        }
        Ok(())
    }
}

/// Signed envelope that binds canonical runtime commit message to signing identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeRuntimeCommitSignedBroadcastEnvelope {
    /// Signer key identifier used by the external custody boundary.
    pub signer_key_id: String,
    /// Canonical runtime commit message that was signed.
    pub message: String,
    /// Signature bytes/encoding for the message.
    pub signature: String,
    /// Signature recovery identifier.
    pub recovery_id: u8,
}

impl KolmeRuntimeCommitSignedBroadcastEnvelope {
    /// Builds a signed broadcast envelope with deterministic validation.
    pub fn new(
        signer_key_id: &str,
        message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        if !is_kolme_valid_signed_envelope_signer_key_id_input_contract(signer_key_id) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signer_key_id",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_signed_envelope_message_input_contract(message) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signed_message",
                reason: "must not be empty",
            });
        }
        if !is_kolme_valid_signed_envelope_signature_input_contract(signature) {
            return Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "signature",
                reason: "must not be empty",
            });
        }
        let (signer_key_id, message, signature) =
            normalize_kolme_runtime_commit_signed_envelope_fields_contract(
                signer_key_id,
                message,
                signature,
            );
        Ok(Self {
            signer_key_id,
            message,
            signature,
            recovery_id,
        })
    }

    /// Returns canonical wire payload used by fork submit profile before normalization.
    pub fn to_wire_payload(&self) -> String {
        render_kolme_signed_envelope_wire_payload_contract(
            self.signer_key_id.as_str(),
            self.message.as_str(),
            self.signature.as_str(),
            self.recovery_id,
        )
    }

    /// Converts the envelope into a Kolme `/broadcast` request payload.
    pub fn to_broadcast_request(
        &self,
    ) -> Result<KolmeApiBroadcastRequest, KolmeRuntimeCommitError> {
        KolmeApiBroadcastRequest::new(
            self.message.as_str(),
            self.signature.as_str(),
            self.recovery_id,
        )
    }
}
