use crate::data_layer_m3_blind_index_search::{
    map_content_retrieval_error_to_m3_projection_error,
    DataLayerM3BlindIndexRetrievalProjectionInput, DataLayerM3RetrievalProjectionRecord,
    DataLayerM3SearchCatalog, DataLayerM3SearchError,
};
use crate::ContentRetrievalRequest;

impl DataLayerM3SearchCatalog {
    pub fn project_blind_index_to_retrieval_requests(
        &self,
        input: DataLayerM3BlindIndexRetrievalProjectionInput,
    ) -> Result<Vec<DataLayerM3RetrievalProjectionRecord>, DataLayerM3SearchError> {
        let DataLayerM3BlindIndexRetrievalProjectionInput {
            blind_index_query,
            requester_did,
            retrieval_scope,
            requested_at_unix,
            message_cids_by_message_id,
        } = input;
        let search_results = self.search_blind_index(blind_index_query)?;

        let mut projection = Vec::with_capacity(search_results.len());
        for record in search_results {
            let cid = message_cids_by_message_id
                .get(record.message_id.as_str())
                .cloned()
                .ok_or_else(|| DataLayerM3SearchError::MissingContentCidForMessage {
                    message_id: record.message_id.clone(),
                })?;
            let retrieval_request = ContentRetrievalRequest::new(
                cid.as_str(),
                requester_did.as_str(),
                retrieval_scope.clone(),
                requested_at_unix,
            )
            .map_err(map_content_retrieval_error_to_m3_projection_error)?;
            projection.push(DataLayerM3RetrievalProjectionRecord {
                message_id: record.message_id,
                cid,
                retrieval_request,
            });
        }
        Ok(projection)
    }
}
