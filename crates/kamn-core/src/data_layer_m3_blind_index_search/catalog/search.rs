use crate::data_layer_m3_blind_index_search::{
    canonical_field_name, resolve_limit, sort_results_deterministically,
    validate_blind_index_token, validate_kamn_did, DataLayerM3BlindIndexQuery,
    DataLayerM3BlindIndexSearchMode, DataLayerM3MessageMetadataRecord, DataLayerM3SearchCatalog,
    DataLayerM3SearchError,
};

impl DataLayerM3SearchCatalog {
    pub fn search_blind_index(
        &self,
        query: DataLayerM3BlindIndexQuery,
    ) -> Result<Vec<DataLayerM3MessageMetadataRecord>, DataLayerM3SearchError> {
        validate_kamn_did(query.owner_did.as_str())?;
        let field_name = canonical_field_name(query.field_name.as_str())?;
        validate_blind_index_token(field_name.as_str(), query.token.as_str())?;
        let limit = resolve_limit(query.limit)?;
        if query.mode != DataLayerM3BlindIndexSearchMode::ExactMatch {
            return Err(DataLayerM3SearchError::UnsupportedBlindIndexSearchMode(
                query.mode,
            ));
        }

        let mut results = self
            .records
            .iter()
            .filter(|record| record.owner_did == query.owner_did)
            .filter(|record| {
                record
                    .blind_indexes
                    .get(field_name.as_str())
                    .is_some_and(|token| token == query.token.trim())
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
