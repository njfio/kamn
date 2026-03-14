use crate::data_layer_m3_blind_index_search::{
    canonical_field_name, resolve_limit, validate_blind_index_token, validate_kamn_did,
    validate_non_empty, DataLayerM3BlindIndexDeterminismDecision,
    DataLayerM3BlindIndexDeterminismInput, DataLayerM3BlindIndexDeterminismReport,
    DataLayerM3BlindIndexQuery, DataLayerM3BlindIndexSearchMode, DataLayerM3SearchCatalog,
    DataLayerM3SearchError, DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE,
    DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE,
};
use std::collections::{BTreeMap, BTreeSet};

impl DataLayerM3SearchCatalog {
    pub fn evaluate_blind_index_determinism(
        &self,
        input: DataLayerM3BlindIndexDeterminismInput,
    ) -> Result<DataLayerM3BlindIndexDeterminismReport, DataLayerM3SearchError> {
        validate_determinism_input(&input)?;
        let field_name = canonical_field_name(input.field_name.as_str())?;
        let limit = resolve_limit(input.limit)?;
        let expected_message_ids = input.baseline_ordered_message_ids.clone();
        let observed_message_ids = self.observed_message_ids(input, field_name, limit)?;
        let missing_message_ids = missing_ids(&expected_message_ids, &observed_message_ids);
        let unexpected_message_ids = missing_ids(&observed_message_ids, &expected_message_ids);
        let out_of_order_message_ids =
            out_of_order_ids(&expected_message_ids, &observed_message_ids);
        let drifted = !missing_message_ids.is_empty()
            || !unexpected_message_ids.is_empty()
            || !out_of_order_message_ids.is_empty();
        let (decision, reason_code) = if drifted {
            (
                DataLayerM3BlindIndexDeterminismDecision::Drifted,
                DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE,
            )
        } else {
            (
                DataLayerM3BlindIndexDeterminismDecision::Stable,
                DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE,
            )
        };
        Ok(DataLayerM3BlindIndexDeterminismReport {
            decision,
            reason_code,
            expected_message_ids,
            observed_message_ids,
            missing_message_ids,
            unexpected_message_ids,
            out_of_order_message_ids,
        })
    }

    fn observed_message_ids(
        &self,
        input: DataLayerM3BlindIndexDeterminismInput,
        field_name: String,
        limit: usize,
    ) -> Result<Vec<String>, DataLayerM3SearchError> {
        self.search_blind_index(DataLayerM3BlindIndexQuery {
            owner_did: input.owner_did,
            field_name,
            token: input.token,
            mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
            limit: Some(limit),
        })
        .map(|records| {
            records
                .into_iter()
                .map(|record| record.message_id)
                .collect()
        })
    }
}

fn validate_determinism_input(
    input: &DataLayerM3BlindIndexDeterminismInput,
) -> Result<(), DataLayerM3SearchError> {
    validate_kamn_did(input.owner_did.as_str())?;
    let field_name = canonical_field_name(input.field_name.as_str())?;
    validate_blind_index_token(field_name.as_str(), input.token.as_str())?;
    if input.baseline_ordered_message_ids.is_empty() {
        return Err(DataLayerM3SearchError::EmptyField(
            "baseline_ordered_message_ids",
        ));
    }
    let mut seen = BTreeSet::new();
    for message_id in &input.baseline_ordered_message_ids {
        validate_non_empty(message_id.as_str(), "baseline_ordered_message_id")?;
        if !seen.insert(message_id.clone()) {
            return Err(DataLayerM3SearchError::DuplicateMessageId(
                message_id.clone(),
            ));
        }
    }
    Ok(())
}

fn missing_ids(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().cloned().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|id| !right.contains(*id))
        .cloned()
        .collect()
}

fn out_of_order_ids(expected: &[String], observed: &[String]) -> Vec<String> {
    let observed_rank = observed
        .iter()
        .enumerate()
        .map(|(rank, message_id)| (message_id.clone(), rank))
        .collect::<BTreeMap<_, _>>();
    expected
        .iter()
        .enumerate()
        .filter_map(|(rank, message_id)| {
            observed_rank
                .get(message_id)
                .filter(|observed_rank| **observed_rank != rank)
                .map(|_| message_id.clone())
        })
        .collect()
}
