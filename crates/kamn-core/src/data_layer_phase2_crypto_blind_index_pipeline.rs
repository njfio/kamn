//! Phase-2 operational pipeline contracts for envelope crypto + blind-index derivation.
//!
//! This module composes existing deterministic contracts from:
//! - `direct_message_crypto` for payload sealing,
//! - `data_layer_m0` for append-only envelope record derivation,
//! - `data_layer_m3_blind_index_search` for blind-index token derivation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::{
    data_layer_m3_compute_blind_index, AgentDid, CanonicalMessageEnvelope,
    DataLayerM0EnvelopeRecord, DataLayerM0RecordInput, DataLayerM0WrappedKey,
    DirectMessageCryptoEngine, DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD,
    DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};

/// One recipient DID and key-reference binding for envelope key wrapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPhase2RecipientEncryptionBinding {
    /// Recipient DID resolved for this binding.
    pub recipient_did: String,
    /// Recipient key-reference used for agreement/wrapping.
    pub recipient_key_ref: String,
}

/// Input envelope for deriving one Phase-2 operational artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPhase2OperationalPipelineRequest {
    /// Canonical envelope payload to seal + derive.
    pub envelope: CanonicalMessageEnvelope,
    /// Sender key-reference used for direct-message sealing.
    pub sender_key_ref: String,
    /// Recipient DID/key-reference bindings.
    pub recipient_bindings: Vec<DataLayerPhase2RecipientEncryptionBinding>,
    /// Blind-index key material used for M3 token derivation.
    pub blind_index_key_material: String,
    /// Field-name/value inputs for blind-index token derivation.
    pub blind_index_fields: BTreeMap<String, String>,
    /// Optional compression dictionary marker.
    pub compression_dict_id: Option<u32>,
    /// Previous hash-chain pointer.
    pub hash_chain_prev: String,
    /// Nonce used for direct-message encryption.
    pub nonce: u64,
}

/// Deterministic Phase-2 operational artifact output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPhase2OperationalPipelineArtifact {
    /// Derived M0 append-only envelope record.
    pub envelope_record: DataLayerM0EnvelopeRecord,
    /// Derived blind-index token map used for persistence/search.
    pub blind_indexes: BTreeMap<String, String>,
}

/// Error taxonomy for Phase-2 operational pipeline derivation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPhase2OperationalPipelineError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Recipient bindings were missing or incomplete for envelope recipients.
    MissingRecipientBindings {
        /// Fail-closed detail.
        detail: String,
    },
    /// Key-reference failed validation.
    InvalidKeyRef {
        /// Field carrying the invalid key-reference.
        field: &'static str,
        /// Fail-closed detail.
        detail: String,
    },
    /// Blind-index token derivation failed.
    BlindIndexDeriveFailed {
        /// Field-name that failed token derivation.
        field_name: String,
        /// Fail-closed detail.
        detail: String,
    },
    /// M0 envelope-record derivation failed.
    EnvelopeRecordDeriveFailed {
        /// Fail-closed detail.
        detail: String,
    },
    /// Encryption failed during payload sealing.
    EncryptionFailed {
        /// Fail-closed detail.
        detail: String,
    },
}

impl fmt::Display for DataLayerPhase2OperationalPipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::MissingRecipientBindings { detail } => {
                write!(formatter, "missing recipient bindings: {detail}")
            }
            Self::InvalidKeyRef { field, detail } => {
                write!(formatter, "invalid key ref for {field}: {detail}")
            }
            Self::BlindIndexDeriveFailed { field_name, detail } => {
                write!(
                    formatter,
                    "blind-index derivation failed for {field_name}: {detail}"
                )
            }
            Self::EnvelopeRecordDeriveFailed { detail } => {
                write!(formatter, "envelope record derivation failed: {detail}")
            }
            Self::EncryptionFailed { detail } => write!(formatter, "encryption failed: {detail}"),
        }
    }
}

impl std::error::Error for DataLayerPhase2OperationalPipelineError {}

/// Derives one deterministic Phase-2 operational artifact.
pub fn data_layer_phase2_build_operational_artifact(
    request: DataLayerPhase2OperationalPipelineRequest,
) -> Result<DataLayerPhase2OperationalPipelineArtifact, DataLayerPhase2OperationalPipelineError> {
    if request.sender_key_ref.trim().is_empty() {
        return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
            "sender_key_ref",
        ));
    }
    if request.blind_index_key_material.trim().is_empty() {
        return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
            "blind_index_key_material",
        ));
    }
    if request.hash_chain_prev.trim().is_empty() {
        return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
            "hash_chain_prev",
        ));
    }
    if request.blind_index_fields.is_empty() {
        return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
            "blind_index_fields",
        ));
    }

    let mut recipient_bindings = request.recipient_bindings;
    if recipient_bindings.is_empty() {
        return Err(
            DataLayerPhase2OperationalPipelineError::MissingRecipientBindings {
                detail: "recipient_bindings must include at least one binding".to_owned(),
            },
        );
    }

    recipient_bindings.sort_by(|left, right| {
        left.recipient_did
            .cmp(&right.recipient_did)
            .then(left.recipient_key_ref.cmp(&right.recipient_key_ref))
    });
    for binding in &recipient_bindings {
        if binding.recipient_did.trim().is_empty() {
            return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
                "recipient_did",
            ));
        }
        AgentDid::parse(binding.recipient_did.as_str()).map_err(|error| {
            DataLayerPhase2OperationalPipelineError::MissingRecipientBindings {
                detail: format!("invalid recipient_did {}: {error}", binding.recipient_did),
            }
        })?;
        if binding.recipient_key_ref.trim().is_empty() {
            return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
                "recipient_key_ref",
            ));
        }
    }

    let binding_dids = recipient_bindings
        .iter()
        .map(|binding| binding.recipient_did.clone())
        .collect::<BTreeSet<_>>();
    for recipient_did in &request.envelope.envelope.to {
        if !binding_dids.contains(recipient_did) {
            return Err(
                DataLayerPhase2OperationalPipelineError::MissingRecipientBindings {
                    detail: format!("missing binding for envelope recipient {recipient_did}"),
                },
            );
        }
    }

    let primary_binding = recipient_bindings.first().ok_or(
        DataLayerPhase2OperationalPipelineError::MissingRecipientBindings {
            detail: "recipient_bindings must include at least one binding".to_owned(),
        },
    )?;
    let mut crypto_engine = DirectMessageCryptoEngine::new(
        request.sender_key_ref.as_str(),
        primary_binding.recipient_key_ref.as_str(),
    )
    .map_err(|error| match error {
        crate::DirectMessageCryptoError::InvalidKeyRef("sender")
        | crate::DirectMessageCryptoError::EmptyKeyRef("sender") => {
            DataLayerPhase2OperationalPipelineError::InvalidKeyRef {
                field: "sender_key_ref",
                detail: error.to_string(),
            }
        }
        crate::DirectMessageCryptoError::InvalidKeyRef("recipient")
        | crate::DirectMessageCryptoError::EmptyKeyRef("recipient") => {
            DataLayerPhase2OperationalPipelineError::InvalidKeyRef {
                field: "recipient_key_ref",
                detail: error.to_string(),
            }
        }
        other => DataLayerPhase2OperationalPipelineError::EncryptionFailed {
            detail: other.to_string(),
        },
    })?;

    let canonical_payload = request.envelope.canonical_payload();
    let ciphertext = crypto_engine
        .encrypt(canonical_payload.as_str(), request.nonce)
        .map_err(
            |error| DataLayerPhase2OperationalPipelineError::EncryptionFailed {
                detail: error.to_string(),
            },
        )?;
    if ciphertext.key_agreement_algorithm != DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM {
        return Err(DataLayerPhase2OperationalPipelineError::EncryptionFailed {
            detail: "unexpected key_agreement_algorithm emitted by direct-message engine"
                .to_owned(),
        });
    }

    let blind_indexes = data_layer_phase2_derive_blind_indexes(
        request.blind_index_key_material.as_str(),
        &request.blind_index_fields,
    )?;
    let wrapped_keys = data_layer_phase2_derive_wrapped_keys(
        request.blind_index_key_material.as_str(),
        request.sender_key_ref.as_str(),
        request.nonce,
        ciphertext.auth_tag.as_str(),
        &recipient_bindings,
    )?;

    let content_size_bytes = canonical_payload.len();
    if content_size_bytes == 0 {
        return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
            "envelope_payload",
        ));
    }
    let envelope_record = DataLayerM0EnvelopeRecord::derive(
        DataLayerM0RecordInput {
            envelope: request.envelope,
            ciphertext,
            wrapped_keys,
            compression_codec: DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD.to_owned(),
            compression_dict_id: request.compression_dict_id,
            content_size_bytes,
            compressed_size_bytes: content_size_bytes,
        },
        request.hash_chain_prev.as_str(),
    )
    .map_err(
        |error| DataLayerPhase2OperationalPipelineError::EnvelopeRecordDeriveFailed {
            detail: error.to_string(),
        },
    )?;

    Ok(DataLayerPhase2OperationalPipelineArtifact {
        envelope_record,
        blind_indexes,
    })
}

fn data_layer_phase2_derive_blind_indexes(
    blind_index_key_material: &str,
    blind_index_fields: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, DataLayerPhase2OperationalPipelineError> {
    let mut tokens = BTreeMap::new();
    for (field_name, field_value) in blind_index_fields {
        if field_name.trim().is_empty() {
            return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
                "blind_index_field_name",
            ));
        }
        if field_value.trim().is_empty() {
            return Err(DataLayerPhase2OperationalPipelineError::EmptyField(
                "blind_index_field_value",
            ));
        }
        let token = data_layer_m3_compute_blind_index(
            blind_index_key_material,
            field_name.as_str(),
            field_value.as_str(),
        )
        .map_err(|error| {
            DataLayerPhase2OperationalPipelineError::BlindIndexDeriveFailed {
                field_name: field_name.clone(),
                detail: error.to_string(),
            }
        })?;
        tokens.insert(field_name.trim().to_owned(), token);
    }
    Ok(tokens)
}

fn data_layer_phase2_derive_wrapped_keys(
    blind_index_key_material: &str,
    sender_key_ref: &str,
    nonce: u64,
    auth_tag: &str,
    recipient_bindings: &[DataLayerPhase2RecipientEncryptionBinding],
) -> Result<Vec<DataLayerM0WrappedKey>, DataLayerPhase2OperationalPipelineError> {
    let mut wrapped_keys = Vec::with_capacity(recipient_bindings.len());
    for binding in recipient_bindings {
        let wrapped_cek = data_layer_m3_compute_blind_index(
            blind_index_key_material,
            "wrapped_cek",
            format!(
                "{}|{}|{}|{}",
                sender_key_ref.trim(),
                binding.recipient_key_ref.trim(),
                nonce,
                auth_tag
            )
            .as_str(),
        )
        .map_err(|error| {
            DataLayerPhase2OperationalPipelineError::BlindIndexDeriveFailed {
                field_name: "wrapped_cek".to_owned(),
                detail: error.to_string(),
            }
        })?;
        wrapped_keys.push(DataLayerM0WrappedKey {
            did: binding.recipient_did.clone(),
            wrapped_cek,
        });
    }
    Ok(wrapped_keys)
}
