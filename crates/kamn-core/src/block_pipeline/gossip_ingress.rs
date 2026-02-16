use super::{
    validation::validate_transport_payload_field_value, BlockPipelineCommitReport,
    BlockPipelineError, CanonicalCommitRecord,
};
use crate::config::NodeRole;
use crate::p2p_transport::PeerGossipFrame;
use crate::transaction::BaselineTransaction;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};

const TOPIC_MESSAGES_LEGACY: &str = "messages";
const TOPIC_MESSAGES_V1: &str = "kamn/messages/v1";
const TOPIC_BLOCKS_LEGACY: &str = "blocks";
const TOPIC_BLOCKS_V1: &str = "kamn/blocks/v1";

/// Encodes a baseline transaction into deterministic transport-candidate wire payload.
pub fn encode_transport_candidate_payload(
    tx: &BaselineTransaction,
) -> Result<String, BlockPipelineError> {
    validate_transport_payload_field_value("id", tx.id.as_str())?;
    validate_transport_payload_field_value("sender", tx.sender.as_str())?;
    validate_transport_payload_field_value("state_hash", tx.state_hash.as_str())?;
    validate_transport_payload_field_value("payload", tx.payload.as_str())?;
    validate_transport_payload_field_value("signature", tx.signature.as_str())?;
    if tx.nonce == 0 {
        return Err(BlockPipelineError::TransportFeed(
            "transport candidate nonce must be positive (transport_candidate_nonce_invalid)"
                .to_owned(),
        ));
    }

    Ok(format!(
        "id={}\nsender={}\nnonce={}\nstate_hash={}\npayload={}\nsignature={}",
        tx.id, tx.sender, tx.nonce, tx.state_hash, tx.payload, tx.signature
    ))
}

/// Decodes deterministic transport-candidate wire payload into baseline transaction.
pub fn decode_transport_candidate_payload(
    payload: &str,
) -> Result<BaselineTransaction, BlockPipelineError> {
    let frame = PeerGossipFrame {
        topic: TOPIC_MESSAGES_V1.to_owned(),
        sender_peer_id: "transport-candidate-decode-source".to_owned(),
        recipient_peer_id: "transport-candidate-decode-target".to_owned(),
        payload: payload.to_owned(),
    };
    match GossipIngressAdapter::decode_frame(&frame) {
        Ok(GossipIngressRecord::Transaction(tx)) => Ok(tx),
        Ok(GossipIngressRecord::BlockCandidate(_)) => Err(BlockPipelineError::TransportFeed(
            "transport candidate decode yielded block candidate payload (transport_candidate_payload_kind_invalid)"
                .to_owned(),
        )),
        Err(error) => Err(BlockPipelineError::TransportFeed(format!(
            "{}:{}",
            error.reason_code(),
            error
        ))),
    }
}

/// Encodes canonical commit candidate into deterministic transport block payload.
pub fn encode_transport_canonical_candidate_payload(
    record: &CanonicalCommitRecord,
) -> Result<String, BlockPipelineError> {
    if record.block_height == 0 {
        return Err(BlockPipelineError::TransportFeed(
            "transport canonical candidate block_height must be positive (transport_candidate_block_height_invalid)"
                .to_owned(),
        ));
    }
    validate_transport_payload_field_value("payload_digest", record.payload_digest.as_str())?;
    if record.transaction_ids.is_empty() {
        return Err(BlockPipelineError::TransportFeed(
            "transport canonical candidate transaction_ids must not be empty (transport_candidate_transaction_ids_invalid)"
                .to_owned(),
        ));
    }
    let mut seen_ids = BTreeSet::new();
    for tx_id in &record.transaction_ids {
        validate_transport_payload_field_value("transaction_id", tx_id.as_str())?;
        if tx_id.contains(',') {
            return Err(BlockPipelineError::TransportFeed(
                "transport canonical candidate transaction_id contains reserved separator ',' (transport_candidate_transaction_id_invalid)"
                    .to_owned(),
            ));
        }
        if !seen_ids.insert(tx_id) {
            return Err(BlockPipelineError::TransportFeed(
                "transport canonical candidate transaction_id is duplicated (transport_candidate_transaction_id_invalid)"
                    .to_owned(),
            ));
        }
    }
    let transaction_ids = record.transaction_ids.join(",");
    Ok(format!(
        "block_height={}\nproducer_role={}\npayload_digest={}\ntransaction_ids={}",
        record.block_height,
        record.producer_role.as_str(),
        record.payload_digest,
        transaction_ids
    ))
}

/// Decodes deterministic transport canonical-candidate payload into canonical commit record.
pub fn decode_transport_canonical_candidate_payload(
    payload: &str,
) -> Result<CanonicalCommitRecord, BlockPipelineError> {
    let frame = PeerGossipFrame {
        topic: TOPIC_BLOCKS_V1.to_owned(),
        sender_peer_id: "transport-candidate-decode-source".to_owned(),
        recipient_peer_id: "transport-candidate-decode-target".to_owned(),
        payload: payload.to_owned(),
    };
    match GossipIngressAdapter::decode_frame(&frame) {
        Ok(GossipIngressRecord::BlockCandidate(record)) => Ok(record),
        Ok(GossipIngressRecord::Transaction(_)) => Err(BlockPipelineError::TransportFeed(
            "transport canonical candidate decode yielded transaction payload (transport_candidate_payload_kind_invalid)"
                .to_owned(),
        )),
        Err(error) => Err(BlockPipelineError::TransportFeed(format!(
            "{}:{}",
            error.reason_code(),
            error
        ))),
    }
}

/// Encodes committed consensus round report into deterministic transport canonical-candidate payload.
pub fn encode_transport_commit_report_payload(
    report: &BlockPipelineCommitReport,
) -> Result<String, BlockPipelineError> {
    let canonical_record = CanonicalCommitRecord::from_commit_report(report);
    encode_transport_canonical_candidate_payload(&canonical_record)
}

/// Decoded gossip ingress record classified by payload intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GossipIngressRecord {
    /// Transaction payload normalized for mempool ingress.
    Transaction(BaselineTransaction),
    /// Canonical block candidate normalized for fork-choice/persistence paths.
    BlockCandidate(CanonicalCommitRecord),
}

impl GossipIngressRecord {
    /// Returns transaction payload when record classification is `Transaction`.
    pub fn into_transaction(self) -> Option<BaselineTransaction> {
        match self {
            Self::Transaction(tx) => Some(tx),
            Self::BlockCandidate(_) => None,
        }
    }

    /// Returns block candidate payload when record classification is `BlockCandidate`.
    pub fn into_block_candidate(self) -> Option<CanonicalCommitRecord> {
        match self {
            Self::Transaction(_) => None,
            Self::BlockCandidate(record) => Some(record),
        }
    }
}

/// Batch decode output for transaction and canonical block candidate payloads.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GossipIngressBatch {
    /// Decoded transaction payloads in input order.
    pub transactions: Vec<BaselineTransaction>,
    /// Decoded canonical block candidates in input order.
    pub canonical_candidates: Vec<CanonicalCommitRecord>,
}

/// Deterministic decode failure for gossip ingress payload normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipIngressError {
    reason_code: &'static str,
    detail: String,
}

impl GossipIngressError {
    fn new(reason_code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            reason_code,
            detail: detail.into(),
        }
    }

    /// Returns deterministic reason code for fail-closed policy checks.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl Display for GossipIngressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.detail, self.reason_code)
    }
}

impl Error for GossipIngressError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GossipIngressTopicKind {
    Transaction,
    Block,
}

/// Deterministic topic+payload ingress adapter for transport-fed pipeline paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GossipIngressAdapter;

impl GossipIngressAdapter {
    /// Decodes one frame into normalized transaction or canonical block candidate record.
    pub fn decode_frame(
        frame: &PeerGossipFrame,
    ) -> Result<GossipIngressRecord, GossipIngressError> {
        let topic_kind = classify_gossip_topic(frame.topic.as_str())?;
        let payload_fields = parse_payload_fields(frame.payload.as_str())?;

        match topic_kind {
            GossipIngressTopicKind::Transaction => decode_transaction_record(&payload_fields),
            GossipIngressTopicKind::Block => decode_block_candidate_record(&payload_fields),
        }
    }

    /// Decodes many frames into transaction and block candidate batches.
    pub fn decode_frames(
        frames: &[PeerGossipFrame],
    ) -> Result<GossipIngressBatch, GossipIngressError> {
        let mut batch = GossipIngressBatch::default();
        for frame in frames {
            match Self::decode_frame(frame)? {
                GossipIngressRecord::Transaction(tx) => batch.transactions.push(tx),
                GossipIngressRecord::BlockCandidate(record) => {
                    batch.canonical_candidates.push(record);
                }
            }
        }
        Ok(batch)
    }
}

fn classify_gossip_topic(topic: &str) -> Result<GossipIngressTopicKind, GossipIngressError> {
    match topic.trim() {
        TOPIC_MESSAGES_LEGACY | TOPIC_MESSAGES_V1 => Ok(GossipIngressTopicKind::Transaction),
        TOPIC_BLOCKS_LEGACY | TOPIC_BLOCKS_V1 => Ok(GossipIngressTopicKind::Block),
        unsupported => Err(GossipIngressError::new(
            "p2p_ingress_topic_unsupported",
            format!("unsupported gossip topic for block pipeline ingress: {unsupported}"),
        )),
    }
}

fn parse_payload_fields(payload: &str) -> Result<BTreeMap<String, String>, GossipIngressError> {
    if payload.trim().is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_empty",
            "gossip ingress payload cannot be empty",
        ));
    }

    let mut fields = BTreeMap::new();
    for raw_line in payload.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let (raw_key, raw_value) = line.split_once('=').ok_or_else(|| {
            GossipIngressError::new(
                "p2p_ingress_payload_line_malformed",
                format!("malformed key/value line: {line}"),
            )
        })?;
        let key = raw_key.trim();
        if key.is_empty() {
            return Err(GossipIngressError::new(
                "p2p_ingress_payload_line_malformed",
                format!("payload key cannot be empty: {line}"),
            ));
        }
        if fields.contains_key(key) {
            return Err(GossipIngressError::new(
                "p2p_ingress_payload_duplicate_field",
                format!("duplicate payload field: {key}"),
            ));
        }
        fields.insert(key.to_owned(), raw_value.trim().to_owned());
    }

    if fields.is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_empty",
            "gossip ingress payload has no key/value fields",
        ));
    }

    Ok(fields)
}

fn required_payload_field<'a>(
    fields: &'a BTreeMap<String, String>,
    field: &'static str,
) -> Result<&'a str, GossipIngressError> {
    let value = fields.get(field).ok_or_else(|| {
        GossipIngressError::new(
            "p2p_ingress_payload_missing_field",
            format!("missing required payload field: {field}"),
        )
    })?;
    if value.trim().is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_missing_field",
            format!("required payload field is empty: {field}"),
        ));
    }
    Ok(value.as_str())
}

fn decode_transaction_record(
    fields: &BTreeMap<String, String>,
) -> Result<GossipIngressRecord, GossipIngressError> {
    let id = required_payload_field(fields, "id")?;
    let sender = required_payload_field(fields, "sender")?;
    let nonce_raw = required_payload_field(fields, "nonce")?;
    let state_hash = required_payload_field(fields, "state_hash")?;
    let payload = required_payload_field(fields, "payload")?;
    let signature = required_payload_field(fields, "signature")?;

    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        GossipIngressError::new(
            "p2p_ingress_payload_nonce_invalid",
            format!("nonce field must be positive integer, found: {nonce_raw}"),
        )
    })?;
    if nonce == 0 {
        return Err(GossipIngressError::new(
            "p2p_ingress_payload_nonce_invalid",
            "nonce field must be positive integer, found: 0",
        ));
    }

    let tx = BaselineTransaction {
        id: id.to_owned(),
        sender: sender.to_owned(),
        nonce,
        payload: payload.to_owned(),
        state_hash: state_hash.to_owned(),
        signature: signature.to_owned(),
    };
    if tx.signature != tx.expected_signature() {
        return Err(GossipIngressError::new(
            "p2p_ingress_tx_signature_invalid",
            format!(
                "transaction signature failed baseline profile validation for {}",
                tx.id
            ),
        ));
    }

    Ok(GossipIngressRecord::Transaction(tx))
}

fn decode_block_candidate_record(
    fields: &BTreeMap<String, String>,
) -> Result<GossipIngressRecord, GossipIngressError> {
    let block_height_raw = required_payload_field(fields, "block_height")?;
    let block_height = block_height_raw.parse::<u64>().map_err(|_| {
        GossipIngressError::new(
            "p2p_ingress_block_height_invalid",
            format!("block_height field must be positive integer, found: {block_height_raw}"),
        )
    })?;
    if block_height == 0 {
        return Err(GossipIngressError::new(
            "p2p_ingress_block_height_invalid",
            "block_height field must be positive integer, found: 0",
        ));
    }

    let producer_role_raw = required_payload_field(fields, "producer_role")?;
    let producer_role = match producer_role_raw {
        "processor" => NodeRole::Processor,
        "listener" => NodeRole::Listener,
        "approver" => NodeRole::Approver,
        other => {
            return Err(GossipIngressError::new(
                "p2p_ingress_block_role_invalid",
                format!("unsupported producer_role field value: {other}"),
            ));
        }
    };

    let payload_digest = required_payload_field(fields, "payload_digest")?;
    let transaction_ids_raw = required_payload_field(fields, "transaction_ids")?;

    let mut seen_ids = BTreeSet::new();
    let mut transaction_ids = Vec::new();
    for tx_id in transaction_ids_raw.split(',').map(|value| value.trim()) {
        if tx_id.is_empty() {
            continue;
        }
        if !seen_ids.insert(tx_id.to_owned()) {
            return Err(GossipIngressError::new(
                "p2p_ingress_block_transaction_ids_invalid",
                format!("duplicate transaction id in block candidate payload: {tx_id}"),
            ));
        }
        transaction_ids.push(tx_id.to_owned());
    }
    if transaction_ids.is_empty() {
        return Err(GossipIngressError::new(
            "p2p_ingress_block_transaction_ids_invalid",
            "transaction_ids field must contain at least one identifier",
        ));
    }

    Ok(GossipIngressRecord::BlockCandidate(CanonicalCommitRecord {
        block_height,
        producer_role,
        payload_digest: payload_digest.to_owned(),
        transaction_ids,
    }))
}
