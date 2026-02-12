//! Audit export contracts for compliance and operator evidence workflows.

use crate::AgentDid;
use std::collections::BTreeSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Domain namespace for an auditable event record.
pub enum AuditDomain {
    /// Message lifecycle and delivery events.
    Messages,
    /// Task lifecycle and assignment events.
    Tasks,
    /// Escrow state and settlement events.
    Escrows,
    /// Reputation signal and score mutation events.
    Reputation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical audit event record captured for export bundles.
pub struct AuditEventRecord {
    /// Logical domain this event belongs to.
    pub domain: AuditDomain,
    /// Stable event identifier in the source domain.
    pub event_id: String,
    /// Event timestamp in canonical UTC string form.
    pub occurred_at: String,
    /// DID of the actor that caused the event.
    pub actor: String,
    /// Action label describing the event mutation.
    pub action: String,
    /// Digest of the payload associated with this event.
    pub payload_digest: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Export wire format for rendered audit bundles.
pub enum AuditExportFormat {
    /// Single JSON object bundle.
    Json,
    /// JSON Lines stream format.
    JsonLines,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Filter constraints applied during audit export selection.
pub struct AuditExportFilter {
    /// Optional domain allowlist. Empty means all domains.
    pub domains: BTreeSet<AuditDomain>,
    /// Optional actor DID allowlist. Empty means all actors.
    pub actor_allowlist: BTreeSet<String>,
    /// Optional lower-bound timestamp (inclusive).
    pub from_inclusive: Option<String>,
    /// Optional upper-bound timestamp (inclusive).
    pub to_inclusive: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Request payload used to materialize an audit export bundle.
pub struct AuditExportRequest {
    /// Stable identifier for the export request.
    pub request_id: String,
    /// DID requesting the export.
    pub requested_by: String,
    /// Request timestamp in canonical UTC string form.
    pub requested_at: String,
    /// Desired export output format.
    pub format: AuditExportFormat,
    /// Selection filter for records included in the bundle.
    pub filter: AuditExportFilter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Manifest metadata emitted with every audit export bundle.
pub struct AuditExportManifest {
    /// Identifier of the originating request.
    pub request_id: String,
    /// DID that requested the export.
    pub requested_by: String,
    /// Timestamp when the export was produced.
    pub exported_at: String,
    /// Format used to render this bundle.
    pub format: AuditExportFormat,
    /// Number of records included in the bundle.
    pub record_count: usize,
    /// Deterministic integrity digest across exported records.
    pub integrity_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Full audit export payload including records and manifest metadata.
pub struct AuditExportBundle {
    /// Exported audit records after filter application.
    pub records: Vec<AuditEventRecord>,
    /// Bundle manifest metadata and integrity digest.
    pub manifest: AuditExportManifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// In-memory audit export engine with exporter authorization controls.
pub struct AuditExportEngine {
    authorized_exporters: BTreeSet<String>,
    events: Vec<AuditEventRecord>,
}

impl AuditExportEngine {
    /// Builds an export engine from an authorized exporter DID set.
    pub fn new(authorized_exporters: Vec<String>) -> Result<Self, AuditExportError> {
        if authorized_exporters.is_empty() {
            return Err(AuditExportError::EmptyAuthorizedExporters);
        }

        let mut exporters = BTreeSet::new();
        for exporter in authorized_exporters {
            validate_did(&exporter)?;
            exporters.insert(exporter);
        }

        Ok(Self {
            authorized_exporters: exporters,
            events: Vec::new(),
        })
    }

    /// Ingests and validates a new audit record into the export store.
    pub fn ingest_event(&mut self, event: AuditEventRecord) -> Result<(), AuditExportError> {
        validate_non_empty("event_id", &event.event_id)?;
        validate_non_empty("occurred_at", &event.occurred_at)?;
        validate_did(&event.actor)?;
        validate_non_empty("action", &event.action)?;
        validate_non_empty("payload_digest", &event.payload_digest)?;
        self.events.push(event);
        Ok(())
    }

    /// Exports a filtered bundle for an authorized requestor DID.
    pub fn export(
        &self,
        request: &AuditExportRequest,
    ) -> Result<AuditExportBundle, AuditExportError> {
        validate_non_empty("request_id", &request.request_id)?;
        validate_did(&request.requested_by)?;
        validate_non_empty("requested_at", &request.requested_at)?;

        if !self.authorized_exporters.contains(&request.requested_by) {
            return Err(AuditExportError::UnauthorizedExporter(
                request.requested_by.clone(),
            ));
        }

        if let (Some(from), Some(to)) = (
            request.filter.from_inclusive.as_deref(),
            request.filter.to_inclusive.as_deref(),
        ) {
            if from > to {
                return Err(AuditExportError::InvalidTimeWindow {
                    from: from.to_owned(),
                    to: to.to_owned(),
                });
            }
        }

        let mut records: Vec<AuditEventRecord> = self
            .events
            .iter()
            .filter(|event| matches_filter(event, &request.filter))
            .cloned()
            .collect();

        records.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then_with(|| left.domain.cmp(&right.domain))
                .then_with(|| left.event_id.cmp(&right.event_id))
                .then_with(|| left.actor.cmp(&right.actor))
                .then_with(|| left.action.cmp(&right.action))
        });

        let canonical = canonical_export_payload(&records);
        let integrity_hash = fnv1a_hex(&canonical);

        Ok(AuditExportBundle {
            manifest: AuditExportManifest {
                request_id: request.request_id.clone(),
                requested_by: request.requested_by.clone(),
                exported_at: request.requested_at.clone(),
                format: request.format,
                record_count: records.len(),
                integrity_hash,
            },
            records,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error taxonomy for audit export validation and authorization failures.
pub enum AuditExportError {
    /// Engine constructor received an empty exporter allowlist.
    EmptyAuthorizedExporters,
    /// A required field was empty after trimming.
    EmptyField(&'static str),
    /// A DID field failed syntactic validation.
    InvalidDid(String),
    /// Time window lower-bound is later than the upper-bound.
    InvalidTimeWindow {
        /// Inclusive lower-bound supplied by the request.
        from: String,
        /// Inclusive upper-bound supplied by the request.
        to: String,
    },
    /// Request DID is not authorized to export audit data.
    UnauthorizedExporter(String),
}

impl fmt::Display for AuditExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAuthorizedExporters => write!(f, "authorized_exporters must not be empty"),
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidTimeWindow { from, to } => {
                write!(f, "invalid time window: from {from} is after to {to}")
            }
            Self::UnauthorizedExporter(value) => write!(f, "unauthorized exporter: {value}"),
        }
    }
}

impl std::error::Error for AuditExportError {}

fn matches_filter(event: &AuditEventRecord, filter: &AuditExportFilter) -> bool {
    if !filter.domains.is_empty() && !filter.domains.contains(&event.domain) {
        return false;
    }
    if !filter.actor_allowlist.is_empty() && !filter.actor_allowlist.contains(&event.actor) {
        return false;
    }
    if let Some(from) = filter.from_inclusive.as_deref() {
        if event.occurred_at.as_str() < from {
            return false;
        }
    }
    if let Some(to) = filter.to_inclusive.as_deref() {
        if event.occurred_at.as_str() > to {
            return false;
        }
    }
    true
}

fn canonical_export_payload(records: &[AuditEventRecord]) -> String {
    let mut output = String::new();
    for record in records {
        output.push_str(match record.domain {
            AuditDomain::Messages => "messages",
            AuditDomain::Tasks => "tasks",
            AuditDomain::Escrows => "escrows",
            AuditDomain::Reputation => "reputation",
        });
        output.push('|');
        output.push_str(&record.event_id);
        output.push('|');
        output.push_str(&record.occurred_at);
        output.push('|');
        output.push_str(&record.actor);
        output.push('|');
        output.push_str(&record.action);
        output.push('|');
        output.push_str(&record.payload_digest);
        output.push('\n');
    }
    output
}

fn fnv1a_hex(value: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), AuditExportError> {
    if value.trim().is_empty() {
        return Err(AuditExportError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), AuditExportError> {
    AgentDid::parse(value).map_err(|error| AuditExportError::InvalidDid(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        AuditDomain, AuditEventRecord, AuditExportEngine, AuditExportError, AuditExportFilter,
        AuditExportFormat, AuditExportRequest,
    };
    use std::collections::BTreeSet;

    #[test]
    fn constructor_requires_authorized_exporter() {
        assert_eq!(
            AuditExportEngine::new(Vec::new()),
            Err(AuditExportError::EmptyAuthorizedExporters)
        );
    }

    #[test]
    fn invalid_time_window_is_rejected() {
        let mut engine = AuditExportEngine::new(vec!["kamn:did:agent:audit-operator".to_owned()])
            .expect("engine should construct");
        engine
            .ingest_event(AuditEventRecord {
                domain: AuditDomain::Messages,
                event_id: "event-1".to_owned(),
                occurred_at: "2026-02-08T10:00:00Z".to_owned(),
                actor: "kamn:did:agent:operator-1".to_owned(),
                action: "MessageSent".to_owned(),
                payload_digest: "sha256:1".to_owned(),
            })
            .expect("event should ingest");

        let request = AuditExportRequest {
            request_id: "req-1".to_owned(),
            requested_by: "kamn:did:agent:audit-operator".to_owned(),
            requested_at: "2026-02-08T10:10:00Z".to_owned(),
            format: AuditExportFormat::Json,
            filter: AuditExportFilter {
                domains: [AuditDomain::Messages].into_iter().collect(),
                actor_allowlist: BTreeSet::new(),
                from_inclusive: Some("2026-02-08T11:00:00Z".to_owned()),
                to_inclusive: Some("2026-02-08T10:00:00Z".to_owned()),
            },
        };

        assert_eq!(
            engine.export(&request),
            Err(AuditExportError::InvalidTimeWindow {
                from: "2026-02-08T11:00:00Z".to_owned(),
                to: "2026-02-08T10:00:00Z".to_owned(),
            })
        );
    }
}
