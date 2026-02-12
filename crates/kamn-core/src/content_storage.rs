//! Content storage adapter contracts and CID/URI mapping helpers.
//!
//! This module defines in-memory reference behavior for storing content payloads,
//! retrieving metadata, and verifying deterministic integrity tags.

use std::collections::BTreeMap;
use std::fmt;

const CID_PREFIX: &str = "kamn:cid:v1:";
const CONTENT_URI_PREFIX: &str = "kamn:content:v1:";
const CID_HASH_HEX_LEN: usize = 16;

/// Stored content payload and metadata returned by `get`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentObject {
    /// Deterministic content identifier.
    pub cid: String,
    /// MIME media type associated with the payload.
    pub media_type: String,
    /// Raw payload bytes.
    pub payload: Vec<u8>,
    /// Deterministic integrity tag for payload verification.
    pub integrity_tag: String,
}

/// Content metadata returned by `head` and `put`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentHead {
    /// Deterministic content identifier.
    pub cid: String,
    /// MIME media type associated with the payload.
    pub media_type: String,
    /// Payload length in bytes.
    pub size_bytes: u64,
    /// Deterministic integrity tag for payload verification.
    pub integrity_tag: String,
}

/// Storage adapter interface for content put/get/head/verify operations.
pub trait ContentStorageAdapter {
    /// Stores payload bytes and returns metadata for the persisted object.
    fn put(&mut self, media_type: &str, payload: &[u8])
        -> Result<ContentHead, ContentStorageError>;
    /// Loads a full content object by CID.
    fn get(&self, cid: &str) -> Result<ContentObject, ContentStorageError>;
    /// Loads only metadata for a CID without payload bytes.
    fn head(&self, cid: &str) -> Result<ContentHead, ContentStorageError>;
    /// Verifies CID and integrity tag against current stored payload.
    fn verify(&self, cid: &str) -> Result<(), ContentStorageError>;
}

/// In-memory reference implementation of `ContentStorageAdapter`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InMemoryContentAdapter {
    objects: BTreeMap<String, StoredObject>,
}

impl InMemoryContentAdapter {
    /// Constructs an empty in-memory content adapter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces payload bytes for a CID without updating integrity metadata.
    ///
    /// This helper is intentionally unsafe for integrity and is used in tests to
    /// simulate storage corruption.
    pub fn replace_payload_unchecked(
        &mut self,
        cid: &str,
        payload: Vec<u8>,
    ) -> Result<(), ContentStorageError> {
        validate_cid(cid)?;
        let object = self
            .objects
            .get_mut(cid)
            .ok_or_else(|| ContentStorageError::NotFound(cid.to_owned()))?;
        object.payload = payload;
        Ok(())
    }
}

impl ContentStorageAdapter for InMemoryContentAdapter {
    fn put(
        &mut self,
        media_type: &str,
        payload: &[u8],
    ) -> Result<ContentHead, ContentStorageError> {
        if media_type.trim().is_empty() {
            return Err(ContentStorageError::EmptyField("media_type"));
        }

        let cid = cid_for_payload(payload);
        let record = StoredObject {
            media_type: media_type.to_owned(),
            payload: payload.to_vec(),
            integrity_tag: integrity_tag_for_payload(payload),
        };
        self.objects.insert(cid.clone(), record.clone());
        Ok(content_head_from_stored(&cid, &record))
    }

    fn get(&self, cid: &str) -> Result<ContentObject, ContentStorageError> {
        validate_cid(cid)?;
        let object = self
            .objects
            .get(cid)
            .ok_or_else(|| ContentStorageError::NotFound(cid.to_owned()))?;
        Ok(ContentObject {
            cid: cid.to_owned(),
            media_type: object.media_type.clone(),
            payload: object.payload.clone(),
            integrity_tag: object.integrity_tag.clone(),
        })
    }

    fn head(&self, cid: &str) -> Result<ContentHead, ContentStorageError> {
        validate_cid(cid)?;
        let object = self
            .objects
            .get(cid)
            .ok_or_else(|| ContentStorageError::NotFound(cid.to_owned()))?;
        Ok(content_head_from_stored(cid, object))
    }

    fn verify(&self, cid: &str) -> Result<(), ContentStorageError> {
        validate_cid(cid)?;
        let object = self
            .objects
            .get(cid)
            .ok_or_else(|| ContentStorageError::NotFound(cid.to_owned()))?;

        let expected_cid = cid_for_payload(&object.payload);
        let expected_integrity_tag = integrity_tag_for_payload(&object.payload);
        if expected_cid != cid || expected_integrity_tag != object.integrity_tag {
            return Err(ContentStorageError::IntegrityMismatch {
                cid: cid.to_owned(),
                expected: expected_integrity_tag,
                found: object.integrity_tag.clone(),
            });
        }

        Ok(())
    }
}

/// Converts a CID into canonical content URI form.
pub fn content_uri_for_cid(cid: &str) -> Result<String, ContentStorageError> {
    validate_cid(cid)?;
    Ok(format!("{CONTENT_URI_PREFIX}{cid}"))
}

/// Extracts and validates a CID from canonical content URI form.
pub fn cid_from_content_uri(uri: &str) -> Result<String, ContentStorageError> {
    let cid = uri
        .strip_prefix(CONTENT_URI_PREFIX)
        .ok_or_else(|| ContentStorageError::InvalidContentUri(uri.to_owned()))?;
    validate_cid(cid)?;
    Ok(cid.to_owned())
}

/// Error surface for content storage parsing and integrity checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentStorageError {
    /// Required string field was empty.
    EmptyField(&'static str),
    /// CID string failed validation.
    InvalidCid(String),
    /// Content URI string failed validation.
    InvalidContentUri(String),
    /// Requested CID was not found in storage.
    NotFound(String),
    /// Stored payload no longer matches expected integrity metadata.
    IntegrityMismatch {
        /// CID that failed integrity verification.
        cid: String,
        /// Expected integrity tag for current payload.
        expected: String,
        /// Integrity tag found in stored metadata.
        found: String,
    },
}

impl fmt::Display for ContentStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidCid(value) => write!(f, "invalid content identifier: {value}"),
            Self::InvalidContentUri(value) => write!(f, "invalid content uri: {value}"),
            Self::NotFound(value) => write!(f, "content not found: {value}"),
            Self::IntegrityMismatch {
                cid,
                expected,
                found,
            } => write!(
                f,
                "content integrity mismatch for {cid}; expected {expected}, found {found}"
            ),
        }
    }
}

impl std::error::Error for ContentStorageError {}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoredObject {
    media_type: String,
    payload: Vec<u8>,
    integrity_tag: String,
}

fn content_head_from_stored(cid: &str, object: &StoredObject) -> ContentHead {
    ContentHead {
        cid: cid.to_owned(),
        media_type: object.media_type.clone(),
        size_bytes: object.payload.len() as u64,
        integrity_tag: object.integrity_tag.clone(),
    }
}

fn cid_for_payload(payload: &[u8]) -> String {
    format!("{CID_PREFIX}{}", fnv1a_hex(payload))
}

fn integrity_tag_for_payload(payload: &[u8]) -> String {
    format!("fnv1a64:{}", fnv1a_hex(payload))
}

fn validate_cid(cid: &str) -> Result<(), ContentStorageError> {
    let Some(digest) = cid.strip_prefix(CID_PREFIX) else {
        return Err(ContentStorageError::InvalidCid(cid.to_owned()));
    };
    if digest.len() != CID_HASH_HEX_LEN || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ContentStorageError::InvalidCid(cid.to_owned()));
    }
    Ok(())
}

fn fnv1a_hex(payload: &[u8]) -> String {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    format!("{acc:016x}")
}

#[cfg(test)]
mod tests {
    use super::{
        cid_for_payload, cid_from_content_uri, content_uri_for_cid, fnv1a_hex, validate_cid,
        ContentStorageError,
    };

    #[test]
    fn cid_for_payload_is_deterministic() {
        let first = cid_for_payload(b"payload");
        let second = cid_for_payload(b"payload");
        assert_eq!(first, second);
    }

    #[test]
    fn validate_cid_rejects_invalid_digest() {
        assert_eq!(
            validate_cid("kamn:cid:v1:nothex"),
            Err(ContentStorageError::InvalidCid(
                "kamn:cid:v1:nothex".to_owned()
            ))
        );
    }

    #[test]
    fn content_uri_round_trip_preserves_cid() {
        let cid = cid_for_payload(b"round-trip");
        let uri = content_uri_for_cid(&cid).expect("uri should be valid");
        let decoded = cid_from_content_uri(&uri).expect("uri should decode");
        assert_eq!(decoded, cid);
    }

    #[test]
    fn fnv1a_hex_produces_expected_width() {
        assert_eq!(fnv1a_hex(b"abc").len(), 16);
    }
}
