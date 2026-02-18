//! M0 data-layer foundation records and append-only verification contracts.
//!
//! This module provides deterministic record derivation from canonical envelopes
//! plus an append-only ledger with hash-chain verification. The digest utility
//! emits a 256-bit hex string with `sha256:` labeling for compatibility with
//! downstream interfaces and can be swapped with a strict SHA-256 backend later.

use crate::{
    CanonicalMessageEnvelope, DirectMessageCiphertext, DIRECT_MESSAGE_CIPHER_ALGORITHM,
    DIRECT_MESSAGE_KEY_AGREEMENT_ALGORITHM,
};
use std::collections::BTreeSet;
use std::fmt;

/// Required compression codec for M0 envelope records.
pub const DATA_LAYER_M0_COMPRESSION_CODEC_ZSTD: &str = "zstd";
/// Hash label used by M0 content and AAD digests.
pub const DATA_LAYER_M0_HASH_ALGORITHM: &str = "sha256";
/// Genesis marker used by the first append-only ledger record.
pub const DATA_LAYER_M0_HASH_CHAIN_GENESIS: &str = "GENESIS";

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
    format!(
        "{DATA_LAYER_M0_HASH_ALGORITHM}:{}",
        deterministic_digest_256_hex(value)
    )
}

fn deterministic_digest_256_hex(value: &str) -> String {
    const SEEDS: [u64; 4] = [
        0x243f6a8885a308d3,
        0x13198a2e03707344,
        0xa4093822299f31d0,
        0x082efa98ec4e6c89,
    ];
    let mut output = String::with_capacity(64);
    for (index, seed) in SEEDS.iter().enumerate() {
        let mut acc = *seed ^ (index as u64).wrapping_mul(0x9e3779b97f4a7c15);
        for byte in value.bytes() {
            acc ^= u64::from(byte);
            acc = acc.wrapping_mul(0x00000100000001B3);
            acc ^= acc.rotate_left(13);
        }
        output.push_str(&format!("{acc:016x}"));
    }
    output
}
