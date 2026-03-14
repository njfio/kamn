use crate::data_layer_m3_blind_index_search::{
    map_content_retrieval_error_to_m3_projection_error,
    DataLayerM3BlindIndexRetrievalProjectionInput, DataLayerM3MessageMetadataRecord,
    DataLayerM3RetrievalProjectionRecord, DataLayerM3SearchCatalog, DataLayerM3SearchError,
};
use crate::ContentRetrievalRequest;
use std::collections::BTreeMap;

impl DataLayerM3SearchCatalog {
    /// Executes blind-index search and projects results to retrieval contracts.
    pub fn project_blind_index_to_retrieval_requests(
        &self,
        input: DataLayerM3BlindIndexRetrievalProjectionInput,
    ) -> Result<Vec<DataLayerM3RetrievalProjectionRecord>, DataLayerM3SearchError> {
        let search_results = self.search_blind_index(input.blind_index_query.clone())?;
        search_results
            .into_iter()
            .map(|record| project_record(&input, record))
            .collect()
    }
}

fn project_record(
    input: &DataLayerM3BlindIndexRetrievalProjectionInput,
    record: DataLayerM3MessageMetadataRecord,
) -> Result<DataLayerM3RetrievalProjectionRecord, DataLayerM3SearchError> {
    let cid = content_cid_for_message(
        &input.message_cids_by_message_id,
        record.message_id.as_str(),
    )?;
    let retrieval_request = ContentRetrievalRequest::new(
        cid.as_str(),
        input.requester_did.as_str(),
        input.retrieval_scope.clone(),
        input.requested_at_unix,
    )
    .map_err(map_content_retrieval_error_to_m3_projection_error)?;
    Ok(DataLayerM3RetrievalProjectionRecord {
        message_id: record.message_id,
        cid,
        retrieval_request,
    })
}

fn content_cid_for_message(
    message_cids_by_message_id: &BTreeMap<String, String>,
    message_id: &str,
) -> Result<String, DataLayerM3SearchError> {
    message_cids_by_message_id
        .get(message_id)
        .cloned()
        .ok_or_else(|| DataLayerM3SearchError::MissingContentCidForMessage {
            message_id: message_id.to_owned(),
        })
}
