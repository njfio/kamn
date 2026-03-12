use super::super::models::*;
use super::super::support::parse_kamn_did;
use crate::ContentLifecycleManager;

impl DataLayerM5EmbeddingRegistry {
    /// Returns owner-scoped retention-due embeddings aligned with content lifecycle policy.
    pub fn retention_due_for_owner(
        &self,
        owner_did: &str,
        now_epoch_seconds: u64,
    ) -> Result<Vec<DataLayerM5RetentionDueCandidate>, DataLayerM5VectorIntegrationError> {
        let owner_did = parse_kamn_did(owner_did)?;
        if now_epoch_seconds == 0 {
            return Err(DataLayerM5VectorIntegrationError::EmptyField("now_epoch_seconds"));
        }
        let owner_records = owner_records(self, owner_did.as_str())?;
        let mut due_candidates = owner_records
            .iter()
            .filter_map(|record| due_candidate(record, now_epoch_seconds))
            .collect::<Vec<_>>();
        due_candidates.sort_by(|left, right| {
            left.due_at_epoch_seconds
                .cmp(&right.due_at_epoch_seconds)
                .then(left.embedding_id.cmp(&right.embedding_id))
                .then(left.message_id.cmp(&right.message_id))
        });
        Ok(due_candidates)
    }
}

fn owner_records<'a>(
    registry: &'a DataLayerM5EmbeddingRegistry,
    owner_did: &str,
) -> Result<&'a [DataLayerM5EmbeddingRecord], DataLayerM5VectorIntegrationError> {
    registry
        .records_by_owner
        .get(owner_did)
        .map(Vec::as_slice)
        .ok_or_else(|| DataLayerM5VectorIntegrationError::OwnerNotFound {
            owner_did: owner_did.to_owned(),
        })
}

fn due_candidate(
    record: &DataLayerM5EmbeddingRecord,
    now_epoch_seconds: u64,
) -> Option<DataLayerM5RetentionDueCandidate> {
    let retention_profile = ContentLifecycleManager::retention_profile(record.retention_class);
    let due_at_epoch_seconds = record
        .created_at_epoch_seconds
        .saturating_add(retention_profile.retain_for_secs);
    (now_epoch_seconds >= due_at_epoch_seconds).then(|| DataLayerM5RetentionDueCandidate {
        owner_did: record.owner_did.clone(),
        embedding_id: record.embedding_id.clone(),
        message_id: record.message_id.clone(),
        retention_class: record.retention_class,
        due_at_epoch_seconds,
        reason_code: DATA_LAYER_M5_RETENTION_DUE_REASON_CODE,
    })
}
