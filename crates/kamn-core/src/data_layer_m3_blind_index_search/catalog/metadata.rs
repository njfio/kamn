use crate::data_layer_m3_blind_index_search::{
    resolve_limit, sort_results_deterministically, validate_kamn_did, validate_non_empty,
    DataLayerM3MessageMetadataRecord, DataLayerM3MetadataQuery, DataLayerM3SearchCatalog,
    DataLayerM3SearchError,
};

impl DataLayerM3SearchCatalog {
    /// Executes one owner-scoped metadata query.
    pub fn search_metadata(
        &self,
        query: DataLayerM3MetadataQuery,
    ) -> Result<Vec<DataLayerM3MessageMetadataRecord>, DataLayerM3SearchError> {
        validate_metadata_query(&query)?;
        let limit = resolve_limit(query.limit)?;
        let mut results = filtered_metadata_results(self.records.as_slice(), &query);
        sort_results_deterministically(&mut results);
        truncate_results(&mut results, limit);
        Ok(results)
    }
}

fn filtered_metadata_results(
    records: &[DataLayerM3MessageMetadataRecord],
    query: &DataLayerM3MetadataQuery,
) -> Vec<DataLayerM3MessageMetadataRecord> {
    records
        .iter()
        .filter(|record| metadata_matches(record, query))
        .cloned()
        .collect()
}

fn metadata_matches(
    record: &DataLayerM3MessageMetadataRecord,
    query: &DataLayerM3MetadataQuery,
) -> bool {
    owner_scope_matches(record, query)
        && did_filters_match(record, query)
        && metadata_filters_match(record, query)
        && timestamp_filters_match(record, query)
}

fn owner_scope_matches(
    record: &DataLayerM3MessageMetadataRecord,
    query: &DataLayerM3MetadataQuery,
) -> bool {
    record.owner_did == query.owner_did
}

fn did_filters_match(
    record: &DataLayerM3MessageMetadataRecord,
    query: &DataLayerM3MetadataQuery,
) -> bool {
    query
        .sender_did
        .as_ref()
        .is_none_or(|sender| record.sender_did == *sender)
        && query
            .recipient_did
            .as_ref()
            .is_none_or(|recipient| record.recipient_did == *recipient)
}

fn metadata_filters_match(
    record: &DataLayerM3MessageMetadataRecord,
    query: &DataLayerM3MetadataQuery,
) -> bool {
    query
        .session_id
        .as_ref()
        .is_none_or(|session| record.session_id.as_ref() == Some(session))
        && query
            .escrow_id
            .as_ref()
            .is_none_or(|escrow| record.escrow_id.as_ref() == Some(escrow))
        && query
            .message_type
            .as_ref()
            .is_none_or(|message_type| record.message_type == *message_type)
}

fn timestamp_filters_match(
    record: &DataLayerM3MessageMetadataRecord,
    query: &DataLayerM3MetadataQuery,
) -> bool {
    query
        .created_after_inclusive
        .is_none_or(|lower| record.created_at_epoch_seconds >= lower)
        && query
            .created_before_inclusive
            .is_none_or(|upper| record.created_at_epoch_seconds <= upper)
}

fn validate_metadata_query(query: &DataLayerM3MetadataQuery) -> Result<(), DataLayerM3SearchError> {
    validate_kamn_did(query.owner_did.as_str())?;
    validate_metadata_dids(query)?;
    validate_metadata_filters(query)?;
    validate_timestamp_bounds(query)
}

fn validate_metadata_dids(query: &DataLayerM3MetadataQuery) -> Result<(), DataLayerM3SearchError> {
    if let Some(sender_did) = query.sender_did.as_deref() {
        validate_kamn_did(sender_did)?;
    }
    if let Some(recipient_did) = query.recipient_did.as_deref() {
        validate_kamn_did(recipient_did)?;
    }
    Ok(())
}

fn validate_metadata_filters(
    query: &DataLayerM3MetadataQuery,
) -> Result<(), DataLayerM3SearchError> {
    if let Some(session_id) = query.session_id.as_deref() {
        validate_non_empty(session_id, "session_id")?;
    }
    if let Some(escrow_id) = query.escrow_id.as_deref() {
        validate_non_empty(escrow_id, "escrow_id")?;
    }
    if let Some(message_type) = query.message_type.as_deref() {
        validate_non_empty(message_type, "message_type")?;
    }
    Ok(())
}

fn validate_timestamp_bounds(
    query: &DataLayerM3MetadataQuery,
) -> Result<(), DataLayerM3SearchError> {
    if let (Some(after), Some(before)) = (
        query.created_after_inclusive,
        query.created_before_inclusive,
    ) {
        if after > before {
            return Err(DataLayerM3SearchError::InvalidTimestampBounds {
                created_after_inclusive: after,
                created_before_inclusive: before,
            });
        }
    }
    Ok(())
}

fn truncate_results(results: &mut Vec<DataLayerM3MessageMetadataRecord>, limit: usize) {
    if results.len() > limit {
        results.truncate(limit);
    }
}
