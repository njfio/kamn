use crate::data_layer_m3_blind_index_search::{
    canonical_field_name, resolve_limit, sort_results_deterministically,
    validate_blind_index_token, validate_kamn_did, DataLayerM3BlindIndexQuery,
    DataLayerM3BlindIndexSearchMode, DataLayerM3MessageMetadataRecord, DataLayerM3SearchCatalog,
    DataLayerM3SearchError,
};

impl DataLayerM3SearchCatalog {
    /// Executes one owner-scoped blind-index query.
    pub fn search_blind_index(
        &self,
        query: DataLayerM3BlindIndexQuery,
    ) -> Result<Vec<DataLayerM3MessageMetadataRecord>, DataLayerM3SearchError> {
        let field_name = validate_search_query(&query)?;
        let limit = resolve_limit(query.limit)?;
        let mut results = exact_match_results(self.records.as_slice(), &query, field_name.as_str());
        sort_results_deterministically(&mut results);
        truncate_results(&mut results, limit);
        Ok(results)
    }
}

fn validate_search_query(
    query: &DataLayerM3BlindIndexQuery,
) -> Result<String, DataLayerM3SearchError> {
    validate_kamn_did(query.owner_did.as_str())?;
    let field_name = canonical_field_name(query.field_name.as_str())?;
    validate_blind_index_token(field_name.as_str(), query.token.as_str())?;
    if query.mode != DataLayerM3BlindIndexSearchMode::ExactMatch {
        return Err(DataLayerM3SearchError::UnsupportedBlindIndexSearchMode(
            query.mode,
        ));
    }
    Ok(field_name)
}

fn exact_match_results(
    records: &[DataLayerM3MessageMetadataRecord],
    query: &DataLayerM3BlindIndexQuery,
    field_name: &str,
) -> Vec<DataLayerM3MessageMetadataRecord> {
    records
        .iter()
        .filter(|record| record.owner_did == query.owner_did)
        .filter(|record| {
            record
                .blind_indexes
                .get(field_name)
                .is_some_and(|token| token == query.token.trim())
        })
        .cloned()
        .collect()
}

fn truncate_results(results: &mut Vec<DataLayerM3MessageMetadataRecord>, limit: usize) {
    if results.len() > limit {
        results.truncate(limit);
    }
}
