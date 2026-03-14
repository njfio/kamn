use crate::data_layer_m3_blind_index_search::{
    resolve_limit, sort_results_deterministically, validate_kamn_did, validate_non_empty,
    DataLayerM3MessageMetadataRecord, DataLayerM3MetadataQuery, DataLayerM3SearchCatalog,
    DataLayerM3SearchError,
};

impl DataLayerM3SearchCatalog {
    pub fn search_metadata(
        &self,
        query: DataLayerM3MetadataQuery,
    ) -> Result<Vec<DataLayerM3MessageMetadataRecord>, DataLayerM3SearchError> {
        validate_metadata_query(&query)?;
        let limit = resolve_limit(query.limit)?;
        let mut results = self
            .records
            .iter()
            .filter(|record| record.owner_did == query.owner_did)
            .filter(|record| {
                query
                    .sender_did
                    .as_ref()
                    .is_none_or(|sender| record.sender_did == *sender)
            })
            .filter(|record| {
                query
                    .recipient_did
                    .as_ref()
                    .is_none_or(|recipient| record.recipient_did == *recipient)
            })
            .filter(|record| {
                query
                    .session_id
                    .as_ref()
                    .is_none_or(|session| record.session_id.as_ref() == Some(session))
            })
            .filter(|record| {
                query
                    .escrow_id
                    .as_ref()
                    .is_none_or(|escrow| record.escrow_id.as_ref() == Some(escrow))
            })
            .filter(|record| {
                query
                    .message_type
                    .as_ref()
                    .is_none_or(|message_type| record.message_type == *message_type)
            })
            .filter(|record| {
                query
                    .created_after_inclusive
                    .is_none_or(|lower| record.created_at_epoch_seconds >= lower)
            })
            .filter(|record| {
                query
                    .created_before_inclusive
                    .is_none_or(|upper| record.created_at_epoch_seconds <= upper)
            })
            .cloned()
            .collect::<Vec<_>>();
        sort_results_deterministically(&mut results);
        if results.len() > limit {
            results.truncate(limit);
        }
        Ok(results)
    }
}

fn validate_metadata_query(query: &DataLayerM3MetadataQuery) -> Result<(), DataLayerM3SearchError> {
    validate_kamn_did(query.owner_did.as_str())?;
    if let Some(sender_did) = query.sender_did.as_deref() {
        validate_kamn_did(sender_did)?;
    }
    if let Some(recipient_did) = query.recipient_did.as_deref() {
        validate_kamn_did(recipient_did)?;
    }
    if let Some(session_id) = query.session_id.as_deref() {
        validate_non_empty(session_id, "session_id")?;
    }
    if let Some(escrow_id) = query.escrow_id.as_deref() {
        validate_non_empty(escrow_id, "escrow_id")?;
    }
    if let Some(message_type) = query.message_type.as_deref() {
        validate_non_empty(message_type, "message_type")?;
    }
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
