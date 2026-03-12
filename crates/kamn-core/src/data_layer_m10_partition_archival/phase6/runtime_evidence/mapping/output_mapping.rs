use kamn_data_layer::data_layer_m10_phase6_runtime_evidence::DataLayerM10Phase6PolicyRuntimeEvidenceBundle;

use crate::data_layer_m10_partition_archival::DataLayerM10Phase6RuntimeEvidenceBundle;

pub(crate) fn map_phase6_runtime_evidence_bundle_from_policy(
    bundle: DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
) -> DataLayerM10Phase6RuntimeEvidenceBundle {
    DataLayerM10Phase6RuntimeEvidenceBundle {
        owner_did: bundle.owner_did,
        cycle_reason_code: bundle.cycle_reason_code,
        trigger_reason_code: bundle.trigger_reason_code,
        budget_reason_code: bundle.budget_reason_code,
        archived_partition_names: bundle.archived_partition_names,
        archived_object_uris: bundle.archived_object_uris,
        due_candidate_count: bundle.due_candidate_count,
        shredded_message_count: bundle.shredded_message_count,
        projection_report_count: bundle.projection_report_count,
        archived_entry_count: bundle.archived_entry_count,
        runtime_total_cycles: bundle.runtime_total_cycles,
        runtime_executed_cycles: bundle.runtime_executed_cycles,
        runtime_deferred_cycles: bundle.runtime_deferred_cycles,
        runtime_fail_closed_cycles: bundle.runtime_fail_closed_cycles,
        runtime_last_successful_tick_epoch_seconds: bundle
            .runtime_last_successful_tick_epoch_seconds,
        runtime_last_observed_now_epoch_seconds: bundle.runtime_last_observed_now_epoch_seconds,
        runtime_last_reason_code: bundle.runtime_last_reason_code,
        reason_code: bundle.reason_code,
    }
}
