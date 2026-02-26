//! M0 data-layer foundation records and append-only verification contracts.
//!
//! This module provides deterministic record derivation from canonical envelopes
//! plus an append-only ledger with hash-chain verification. The digest utility
//! emits strict SHA-256 digests with `sha256:` labeling for compatibility with
//! downstream interfaces.

use crate::{
    data_layer_hashing::tagged_sha256, CanonicalMessageEnvelope, DirectMessageCiphertext,
    DIRECT_MESSAGE_CIPHER_ALGORITHM, DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
use std::collections::BTreeSet;
use std::fmt;

/// Required compression codec for M0 envelope records.
pub const DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD: &str = "zstd";
/// Hash label used by M0 content and AAD digests.
pub const DATA_LAYER_M0_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker used by the first append-only ledger record.
pub const DATA_LAYER_M0_HASH_CHAIN_GENESIS: &str = "GENESIS";
/// Conformance-matrix decision reason when all invariants match expectations.
pub const DATA_LAYER_M0_CONFORMANCE_MATRIX_STABLE_REASON_CODE: &str = "m0_conformance_stable";
/// Conformance-matrix decision reason when at least one invariant drifts.
pub const DATA_LAYER_M0_CONFORMANCE_MATRIX_DRIFT_REASON_CODE: &str =
    "m0_conformance_drift_detected";

/// Wrapped CEK entry bound to one authorized DID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM0WrappedKey {
    /// Recipient DID for this wrapped CEK.
    pub did: String,
    /// Wrapped CEK payload.
    pub wrapped_cek: String,
}

/// Input payload for deriving one append-only M0 envelope record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM0RecordInput {
    /// Canonical envelope metadata/body/proof.
    pub envelope: CanonicalMessageEnvelope,
    /// Encrypted payload + direct-message crypto metadata.
    pub ciphertext: DirectMessageCiphertext,
    /// Wrapped CEK entries for authorized readers.
    pub wrapped_keys: Vec<DataLayerM0WrappedKey>,
    /// Compression codec marker (must be zstd for M0).
    pub compression_codec: String,
    /// Optional dictionary identifier used by compression.
    pub compression_dict_id: Option<u32>,
    /// Uncompressed envelope size.
    pub content_size_bytes: usize,
    /// Stored compressed size.
    pub compressed_size_bytes: usize,
}

/// Stored append-only M0 message record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM0EnvelopeRecord {
    /// Message identifier (mirrors `envelope.id`).
    pub message_id: String,
    /// Content hash for full canonical envelope storage payload.
    pub content_hash: String,
    /// Previous content hash chain pointer.
    pub hash_chain_prev: String,
    /// Sender DID.
    pub sender_did: String,
    /// Canonically sorted recipients.
    pub recipient_dids: Vec<String>,
    /// Message-type label.
    pub message_type: String,
    /// Serialized ciphertext payload.
    pub envelope_ciphertext: String,
    /// Envelope nonce.
    pub envelope_nonce: u64,
    /// Hash over canonical AAD metadata.
    pub envelope_aad_hash: String,
    /// Canonically sorted wrapped CEKs by DID.
    pub wrapped_keys: Vec<DataLayerM0WrappedKey>,
    /// Compression codec marker.
    pub compression_codec: String,
    /// Optional compression dictionary marker.
    pub compression_dict_id: Option<u32>,
    /// Uncompressed size bytes.
    pub content_size_bytes: usize,
    /// Compressed size bytes.
    pub compressed_size_bytes: usize,
}

impl DataLayerM0EnvelopeRecord {
    /// Derives one deterministic M0 record from canonical envelope and ciphertext metadata.
    pub fn derive(
        input: DataLayerM0RecordInput,
        hash_chain_prev: &str,
    ) -> Result<Self, DataLayerM0Error> {
        if hash_chain_prev.trim().is_empty() {
            return Err(DataLayerM0Error::EmptyField("hash_chain_prev"));
        }

        input
            .envelope
            .validate()
            .map_err(|error| DataLayerM0Error::InvalidEnvelope(error.to_string()))?;

        validate_wrapped_keys(&input.wrapped_keys)?;
        validate_compression(
            &input.compression_codec,
            input.content_size_bytes,
            input.compressed_size_bytes,
        )?;
        validate_ciphertext(&input.ciphertext)?;

        let mut recipient_dids = input.envelope.envelope.to.clone();
        recipient_dids.sort();

        let mut wrapped_keys = input.wrapped_keys.clone();
        wrapped_keys.sort_by(|left, right| {
            left.did
                .cmp(&right.did)
                .then(left.wrapped_cek.cmp(&right.wrapped_cek))
        });

        let aad_canonical = canonical_aad_payload(
            &input.envelope.envelope.from,
            &recipient_dids,
            &input.envelope.envelope.created,
            &input.envelope.envelope.expires,
            &input.envelope.header.message_type,
        );
        let envelope_aad_hash = tagged_digest(&aad_canonical);
        let wrapped_key_payload = wrapped_keys
            .iter()
            .map(|entry| format!("{}={}", entry.did, entry.wrapped_cek))
            .collect::<Vec<_>>()
            .join("|");
        let dict_marker = input
            .compression_dict_id
            .map(|value| value.to_string())
            .unwrap_or_default();

        let canonical_storage_payload = format!(
            "{}|aad:{}|cipher:{}|nonce:{}|auth:{}|wrapped:{}|codec:{}|dict:{}|sizes:{}:{}|prev:{}",
            input.envelope.canonical_payload(),
            envelope_aad_hash,
            input.ciphertext.ciphertext,
            input.ciphertext.nonce,
            input.ciphertext.auth_tag,
            wrapped_key_payload,
            input.compression_codec,
            dict_marker,
            input.content_size_bytes,
            input.compressed_size_bytes,
            hash_chain_prev,
        );
        let content_hash = tagged_digest(&canonical_storage_payload);

        Ok(Self {
            message_id: input.envelope.envelope.id.clone(),
            content_hash,
            hash_chain_prev: hash_chain_prev.to_owned(),
            sender_did: input.envelope.envelope.from,
            recipient_dids,
            message_type: input.envelope.header.message_type,
            envelope_ciphertext: input.ciphertext.ciphertext,
            envelope_nonce: input.ciphertext.nonce,
            envelope_aad_hash,
            wrapped_keys,
            compression_codec: input.compression_codec,
            compression_dict_id: input.compression_dict_id,
            content_size_bytes: input.content_size_bytes,
            compressed_size_bytes: input.compressed_size_bytes,
        })
    }
}

/// Append-only in-memory ledger for M0 message records.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM0AppendOnlyLedger {
    records: Vec<DataLayerM0EnvelopeRecord>,
    seen_message_ids: BTreeSet<String>,
}

impl DataLayerM0AppendOnlyLedger {
    /// Creates an empty M0 ledger.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a record derived from the provided input.
    pub fn append(
        &mut self,
        input: DataLayerM0RecordInput,
    ) -> Result<DataLayerM0EnvelopeRecord, DataLayerM0Error> {
        let message_id = input.envelope.envelope.id.clone();
        if self.seen_message_ids.contains(&message_id) {
            return Err(DataLayerM0Error::DuplicateMessageId(message_id));
        }

        let hash_chain_prev = self
            .records
            .last()
            .map(|record| record.content_hash.clone())
            .unwrap_or_else(|| DATA_LAYER_M0_HASH_CHAIN_GENESIS.to_owned());

        let record = DataLayerM0EnvelopeRecord::derive(input, &hash_chain_prev)?;
        self.seen_message_ids.insert(record.message_id.clone());
        self.records.push(record.clone());
        Ok(record)
    }

    /// Returns the immutable append order.
    pub fn records(&self) -> &[DataLayerM0EnvelopeRecord] {
        &self.records
    }

    /// Verifies hash-chain continuity for the full append-only sequence.
    pub fn verify_hash_chain(&self) -> Result<(), DataLayerM0Error> {
        let mut expected_prev = DATA_LAYER_M0_HASH_CHAIN_GENESIS.to_owned();
        for (position, record) in self.records.iter().enumerate() {
            if record.hash_chain_prev != expected_prev {
                return Err(DataLayerM0Error::InvalidHashChainLink {
                    position,
                    expected_prev,
                    found_prev: record.hash_chain_prev.clone(),
                });
            }
            expected_prev = record.content_hash.clone();
        }
        Ok(())
    }

    /// Replaces one record content hash without recomputing chain links.
    ///
    /// This helper intentionally bypasses integrity checks for deterministic
    /// tamper-detection tests.
    pub fn replace_content_hash_unchecked(
        &mut self,
        message_id: &str,
        content_hash: &str,
    ) -> Result<(), DataLayerM0Error> {
        if content_hash.trim().is_empty() {
            return Err(DataLayerM0Error::EmptyField("content_hash"));
        }

        let record = self
            .records
            .iter_mut()
            .find(|entry| entry.message_id == message_id)
            .ok_or_else(|| DataLayerM0Error::NotFound(message_id.to_owned()))?;
        record.content_hash = content_hash.to_owned();
        Ok(())
    }
}

/// M0 invariant categories tracked by conformance matrix contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM0ConformanceInvariant {
    /// Content/AAD hash determinism for envelope-crypto projection.
    EnvelopeCryptoDeterministic,
    /// Duplicate-message rejection in append-only ledger operations.
    AppendOnlyDuplicateRejected,
    /// Hash-chain tamper detection in append-order verification.
    HashChainTamperDetected,
}

/// One conformance case input for M0 invariant matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM0ConformanceMatrixCase {
    /// Stable case identifier.
    pub case_id: String,
    /// Invariant category under evaluation.
    pub invariant: DataLayerM0ConformanceInvariant,
    /// Observed pass/fail result.
    pub observed_passed: bool,
    /// Expected pass/fail result.
    pub expected_passed: bool,
}

/// Per-case conformance matrix evidence entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM0ConformanceMatrixEvidence {
    /// Stable case identifier.
    pub case_id: String,
    /// Invariant category.
    pub invariant: DataLayerM0ConformanceInvariant,
    /// Observed pass/fail result.
    pub observed_passed: bool,
    /// Expected pass/fail result.
    pub expected_passed: bool,
    /// Whether the case drifted from expectation.
    pub mismatch: bool,
}

/// Aggregate decision for M0 conformance matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM0ConformanceMatrixDecision {
    /// All matrix cases matched expected outcomes.
    Stable {
        /// Stable decision reason marker.
        reason_code: &'static str,
    },
    /// At least one matrix case drifted from expected outcomes.
    DriftDetected {
        /// Stable decision reason marker.
        reason_code: &'static str,
    },
}

/// Aggregate conformance-matrix report for M0 invariants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM0ConformanceMatrixReport {
    /// Aggregate decision.
    pub decision: DataLayerM0ConformanceMatrixDecision,
    /// Per-case evidence entries in input order.
    pub evidence: Vec<DataLayerM0ConformanceMatrixEvidence>,
}

/// Evaluates deterministic conformance matrix outcomes for M0 invariants.
pub fn evaluate_data_layer_m0_conformance_matrix(
    cases: &[DataLayerM0ConformanceMatrixCase],
) -> Result<DataLayerM0ConformanceMatrixReport, DataLayerM0Error> {
    if cases.is_empty() {
        return Err(DataLayerM0Error::InvalidConformanceMatrixInput("cases"));
    }

    let mut evidence = Vec::with_capacity(cases.len());
    for case in cases {
        if case.case_id.trim().is_empty() {
            return Err(DataLayerM0Error::InvalidConformanceMatrixInput("case_id"));
        }
        evidence.push(DataLayerM0ConformanceMatrixEvidence {
            case_id: case.case_id.clone(),
            invariant: case.invariant,
            observed_passed: case.observed_passed,
            expected_passed: case.expected_passed,
            mismatch: case.observed_passed != case.expected_passed,
        });
    }

    let decision = if evidence.iter().all(|entry| !entry.mismatch) {
        DataLayerM0ConformanceMatrixDecision::Stable {
            reason_code: DATA_LAYER_M0_CONFORMANCE_MATRIX_STABLE_REASON_CODE,
        }
    } else {
        DataLayerM0ConformanceMatrixDecision::DriftDetected {
            reason_code: DATA_LAYER_M0_CONFORMANCE_MATRIX_DRIFT_REASON_CODE,
        }
    };

    Ok(DataLayerM0ConformanceMatrixReport { decision, evidence })
}

/// Error taxonomy for M0 record derivation and append-only verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM0Error {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Envelope validation failed.
    InvalidEnvelope(String),
    /// Wrapped keys were missing or malformed.
    InvalidWrappedKey(&'static str),
    /// Wrapped-key set must not be empty.
    EmptyWrappedKeys,
    /// Compression codec did not match M0 contract.
    InvalidCompressionCodec(String),
    /// Compression size constraints failed.
    InvalidCompressionSize {
        /// Uncompressed bytes.
        content_size_bytes: usize,
        /// Compressed bytes.
        compressed_size_bytes: usize,
    },
    /// Ciphertext metadata did not satisfy direct-message contract.
    InvalidCiphertextMetadata(&'static str),
    /// Duplicate message id append was attempted.
    DuplicateMessageId(String),
    /// Hash-chain continuity failed for one record.
    InvalidHashChainLink {
        /// Zero-based record position.
        position: usize,
        /// Expected previous hash.
        expected_prev: String,
        /// Found previous hash.
        found_prev: String,
    },
    /// Conformance-matrix input failed fail-closed validation.
    InvalidConformanceMatrixInput(&'static str),
    /// Message id not present in ledger.
    NotFound(String),
}

impl fmt::Display for DataLayerM0Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidEnvelope(error) => write!(f, "invalid canonical envelope: {error}"),
            Self::InvalidWrappedKey(field) => write!(f, "wrapped key {field} must not be empty"),
            Self::EmptyWrappedKeys => write!(f, "wrapped keys must contain at least one entry"),
            Self::InvalidCompressionCodec(codec) => {
                write!(f, "invalid compression codec: {codec}")
            }
            Self::InvalidCompressionSize {
                content_size_bytes,
                compressed_size_bytes,
            } => write!(
                f,
                "invalid compression sizes: content={content_size_bytes}, compressed={compressed_size_bytes}"
            ),
            Self::InvalidCiphertextMetadata(field) => {
                write!(f, "invalid ciphertext metadata: {field}")
            }
            Self::DuplicateMessageId(message_id) => {
                write!(f, "duplicate message id: {message_id}")
            }
            Self::InvalidHashChainLink {
                position,
                expected_prev,
                found_prev,
            } => write!(
                f,
                "hash-chain link mismatch at position {position}: expected {expected_prev}, found {found_prev}"
            ),
            Self::InvalidConformanceMatrixInput(field) => {
                write!(f, "invalid conformance matrix input: {field}")
            }
            Self::NotFound(message_id) => write!(f, "message id not found: {message_id}"),
        }
    }
}

impl std::error::Error for DataLayerM0Error {}

fn validate_wrapped_keys(wrapped_keys: &[DataLayerM0WrappedKey]) -> Result<(), DataLayerM0Error> {
    if wrapped_keys.is_empty() {
        return Err(DataLayerM0Error::EmptyWrappedKeys);
    }
    for entry in wrapped_keys {
        if entry.did.trim().is_empty() {
            return Err(DataLayerM0Error::InvalidWrappedKey("did"));
        }
        if entry.wrapped_cek.trim().is_empty() {
            return Err(DataLayerM0Error::InvalidWrappedKey("wrapped_cek"));
        }
    }
    Ok(())
}

fn validate_compression(
    compression_codec: &str,
    content_size_bytes: usize,
    compressed_size_bytes: usize,
) -> Result<(), DataLayerM0Error> {
    if compression_codec != DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD {
        return Err(DataLayerM0Error::InvalidCompressionCodec(
            compression_codec.to_owned(),
        ));
    }
    if content_size_bytes == 0
        || compressed_size_bytes == 0
        || compressed_size_bytes > content_size_bytes
    {
        return Err(DataLayerM0Error::InvalidCompressionSize {
            content_size_bytes,
            compressed_size_bytes,
        });
    }
    Ok(())
}

fn validate_ciphertext(ciphertext: &DirectMessageCiphertext) -> Result<(), DataLayerM0Error> {
    if ciphertext.key_agreement_algorithm != DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM {
        return Err(DataLayerM0Error::InvalidCiphertextMetadata(
            "key_agreement_algorithm",
        ));
    }
    if ciphertext.cipher_algorithm != DIRECT_MESSAGE_CIPHER_ALGORITHM {
        return Err(DataLayerM0Error::InvalidCiphertextMetadata(
            "cipher_algorithm",
        ));
    }
    if ciphertext.nonce == 0 {
        return Err(DataLayerM0Error::InvalidCiphertextMetadata("nonce"));
    }
    if ciphertext.ciphertext.trim().is_empty() {
        return Err(DataLayerM0Error::InvalidCiphertextMetadata("ciphertext"));
    }
    if ciphertext.auth_tag.trim().is_empty() {
        return Err(DataLayerM0Error::InvalidCiphertextMetadata("auth_tag"));
    }
    Ok(())
}

fn canonical_aad_payload(
    sender_did: &str,
    recipient_dids: &[String],
    created: &str,
    expires: &str,
    message_type: &str,
) -> String {
    format!(
        "{}|{}|{}|{}|{}",
        sender_did,
        recipient_dids.join(","),
        created,
        expires,
        message_type
    )
}

fn tagged_digest(value: &str) -> String {
    tagged_sha256(value, DATA_LAYER_M0_HASH_ALGORITHM)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AttachmentRef, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata, EnvelopeProof,
    };
    use std::collections::BTreeMap;

    fn fixture_envelope(message_id: &str, nonce: u64) -> CanonicalMessageEnvelope {
        let mut body = BTreeMap::new();
        body.insert("operation".to_owned(), "store".to_owned());
        body.insert("payload".to_owned(), format!("payload-{message_id}"));

        CanonicalMessageEnvelope {
            envelope: EnvelopeMetadata {
                id: message_id.to_owned(),
                type_name: crate::CANONICAL_MESSAGE_ENVELOPE_TYPE.to_owned(),
                from: "kamn:did:agent:sender-1".to_owned(),
                to: vec![
                    "kamn:did:agent:recipient-2".to_owned(),
                    "kamn:did:agent:recipient-1".to_owned(),
                ],
                created: "2026-02-07T20:15:30.123Z".to_owned(),
                expires: "2026-02-07T20:45:30.123Z".to_owned(),
                thread_id: Some("urn:uuid:thread-1".to_owned()),
                parent_id: None,
                nonce,
            },
            header: EnvelopeHeader {
                message_type: "Request".to_owned(),
                priority: "Elevated".to_owned(),
                content_type: "application/json".to_owned(),
                encryption: EnvelopeEncryption {
                    algorithm: crate::CANONICAL_ENCRYPTION_ALGORITHM.to_owned(),
                    recipient_keys: vec![
                        "kamn:did:agent:recipient-2#key-agreement-1".to_owned(),
                        "kamn:did:agent:recipient-1#key-agreement-1".to_owned(),
                    ],
                },
            },
            body,
            attachments: vec![AttachmentRef {
                id: "attachment-1".to_owned(),
                media_type: "application/json".to_owned(),
                uri: "ipfs://Qm123".to_owned(),
            }],
            proof: EnvelopeProof {
                type_name: "Ed25519Signature2020".to_owned(),
                created: "2026-02-07T20:15:30.123Z".to_owned(),
                verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
                proof_purpose: crate::CANONICAL_PROOF_PURPOSE.to_owned(),
                proof_value: "z58proof".to_owned(),
            },
        }
    }

    fn fixture_ciphertext(nonce: u64) -> DirectMessageCiphertext {
        DirectMessageCiphertext {
            key_agreement_algorithm: DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM.to_owned(),
            cipher_algorithm: DIRECT_MESSAGE_CIPHER_ALGORITHM.to_owned(),
            sender_key_ref: "kamn:did:agent:sender-1#key-agreement-1".to_owned(),
            recipient_key_ref: "kamn:did:agent:recipient-1#key-agreement-1".to_owned(),
            nonce,
            ciphertext: format!("cipher-{nonce}"),
            auth_tag: format!("auth-{nonce}"),
        }
    }

    fn fixture_input(message_id: &str, nonce: u64) -> DataLayerM0RecordInput {
        DataLayerM0RecordInput {
            envelope: fixture_envelope(message_id, nonce),
            ciphertext: fixture_ciphertext(nonce),
            wrapped_keys: vec![
                DataLayerM0WrappedKey {
                    did: "kamn:did:agent:recipient-2".to_owned(),
                    wrapped_cek: "cek-b".to_owned(),
                },
                DataLayerM0WrappedKey {
                    did: "kamn:did:agent:recipient-1".to_owned(),
                    wrapped_cek: "cek-a".to_owned(),
                },
            ],
            compression_codec: DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD.to_owned(),
            compression_dict_id: Some(7),
            content_size_bytes: 128,
            compressed_size_bytes: 96,
        }
    }

    #[test]
    fn unit_append_builds_expected_chain_links_and_sorts_projection_fields() {
        let mut ledger = DataLayerM0AppendOnlyLedger::new();
        let first = ledger
            .append(fixture_input("urn:uuid:msg-1", 41))
            .expect("first append should succeed");
        let second = ledger
            .append(fixture_input("urn:uuid:msg-2", 42))
            .expect("second append should succeed");

        assert_eq!(first.hash_chain_prev, DATA_LAYER_M0_HASH_CHAIN_GENESIS);
        assert_eq!(second.hash_chain_prev, first.content_hash);
        assert_eq!(
            first.recipient_dids,
            vec![
                "kamn:did:agent:recipient-1".to_owned(),
                "kamn:did:agent:recipient-2".to_owned()
            ]
        );
        assert_eq!(
            first.wrapped_keys,
            vec![
                DataLayerM0WrappedKey {
                    did: "kamn:did:agent:recipient-1".to_owned(),
                    wrapped_cek: "cek-a".to_owned(),
                },
                DataLayerM0WrappedKey {
                    did: "kamn:did:agent:recipient-2".to_owned(),
                    wrapped_cek: "cek-b".to_owned(),
                }
            ]
        );
        assert_eq!(ledger.verify_hash_chain(), Ok(()));
    }

    #[test]
    fn regression_append_rejects_duplicate_message_id() {
        let mut ledger = DataLayerM0AppendOnlyLedger::new();
        let _ = ledger
            .append(fixture_input("urn:uuid:msg-1", 41))
            .expect("first append should succeed");
        let duplicate = ledger.append(fixture_input("urn:uuid:msg-1", 99));
        assert_eq!(
            duplicate,
            Err(DataLayerM0Error::DuplicateMessageId(
                "urn:uuid:msg-1".to_owned()
            ))
        );
    }

    #[test]
    fn regression_verify_hash_chain_detects_tampered_previous_link() {
        let mut ledger = DataLayerM0AppendOnlyLedger::new();
        let first = ledger
            .append(fixture_input("urn:uuid:msg-1", 41))
            .expect("first append should succeed");
        let _second = ledger
            .append(fixture_input("urn:uuid:msg-2", 42))
            .expect("second append should succeed");

        ledger
            .replace_content_hash_unchecked("urn:uuid:msg-1", "sha256:deadbeef")
            .expect("tamper helper should succeed");

        assert_eq!(
            ledger.verify_hash_chain(),
            Err(DataLayerM0Error::InvalidHashChainLink {
                position: 1,
                expected_prev: "sha256:deadbeef".to_owned(),
                found_prev: first.content_hash,
            })
        );
    }

    #[test]
    fn unit_conformance_matrix_projects_stable_and_drift_decisions() {
        let stable_cases = vec![
            DataLayerM0ConformanceMatrixCase {
                case_id: "c-01".to_owned(),
                invariant: DataLayerM0ConformanceInvariant::EnvelopeCryptoDeterministic,
                observed_passed: true,
                expected_passed: true,
            },
            DataLayerM0ConformanceMatrixCase {
                case_id: "c-02".to_owned(),
                invariant: DataLayerM0ConformanceInvariant::AppendOnlyDuplicateRejected,
                observed_passed: false,
                expected_passed: false,
            },
        ];
        let stable_report =
            evaluate_data_layer_m0_conformance_matrix(&stable_cases).expect("stable report");
        assert_eq!(
            stable_report.decision,
            DataLayerM0ConformanceMatrixDecision::Stable {
                reason_code: DATA_LAYER_M0_CONFORMANCE_MATRIX_STABLE_REASON_CODE,
            }
        );

        let drift_cases = vec![DataLayerM0ConformanceMatrixCase {
            case_id: "c-03".to_owned(),
            invariant: DataLayerM0ConformanceInvariant::HashChainTamperDetected,
            observed_passed: false,
            expected_passed: true,
        }];
        let drift_report =
            evaluate_data_layer_m0_conformance_matrix(&drift_cases).expect("drift report");
        assert_eq!(
            drift_report.decision,
            DataLayerM0ConformanceMatrixDecision::DriftDetected {
                reason_code: DATA_LAYER_M0_CONFORMANCE_MATRIX_DRIFT_REASON_CODE,
            }
        );
        assert_eq!(drift_report.evidence.len(), 1);
        assert!(drift_report.evidence[0].mismatch);
    }
}
