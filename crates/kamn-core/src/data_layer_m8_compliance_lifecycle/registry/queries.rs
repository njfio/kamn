use super::super::{
    lifecycle::authorize_owner_scope, DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry,
    DataLayerM8OwnerScopeQuery, DataLayerM8RetentionDueCandidate,
    DATA_LAYER_M8_RETENTION_DUE_REASON_CODE,
};

pub(super) fn retention_due_for_owner(
    registry: &DataLayerM8ComplianceRegistry,
    query: DataLayerM8OwnerScopeQuery,
    now_epoch_seconds: u64,
) -> Result<Vec<DataLayerM8RetentionDueCandidate>, DataLayerM8ComplianceError> {
    let owner_did =
        authorize_owner_scope(query.requester_owner_did.as_str(), query.owner_did.as_str())?;
    if now_epoch_seconds == 0 {
        return Err(DataLayerM8ComplianceError::EmptyField("now_epoch_seconds"));
    }
    let owner_records = registry.owner_records_or_error(owner_did.as_str())?;
    let mut candidates = collect_due_candidates(owner_records, now_epoch_seconds);
    candidates.sort_by(|left, right| {
        left.due_at_epoch_seconds
            .cmp(&right.due_at_epoch_seconds)
            .then(left.message_id.cmp(&right.message_id))
    });
    Ok(candidates)
}

fn collect_due_candidates(
    owner_records: &[super::super::DataLayerM8MessageRecord],
    now_epoch_seconds: u64,
) -> Vec<DataLayerM8RetentionDueCandidate> {
    owner_records
        .iter()
        .filter_map(|record| build_due_candidate(record, now_epoch_seconds))
        .collect()
}

fn build_due_candidate(
    record: &super::super::DataLayerM8MessageRecord,
    now_epoch_seconds: u64,
) -> Option<DataLayerM8RetentionDueCandidate> {
    if record.shredded_at_epoch_seconds.is_some() || record.legal_hold_active {
        return None;
    }
    let base_window_seconds = record.retention_class.retention_window_seconds()?;
    let due_at_epoch_seconds = record
        .created_at_epoch_seconds
        .saturating_add(base_window_seconds)
        .saturating_add(record.retention_extension_seconds);
    (now_epoch_seconds >= due_at_epoch_seconds).then(|| DataLayerM8RetentionDueCandidate {
        owner_did: record.owner_did.clone(),
        message_id: record.message_id.clone(),
        retention_class: record.retention_class,
        due_at_epoch_seconds,
        reason_code: DATA_LAYER_M8_RETENTION_DUE_REASON_CODE,
    })
}
