use super::constants::*;
use super::gate_support::*;
use super::models::*;
use super::*;
fn missing_multi_host_decision(
    reason_code: &'static str,
) -> LivePostgresMultiHostPrerequisiteDecision {
    LivePostgresMultiHostPrerequisiteDecision {
        reason_code,
        reason_taxonomy_version: LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION,
        host_pair_csv: None,
    }
}

fn ready_host_pair_csv() -> Result<String, LivePostgresMultiHostPrerequisiteDecision> {
    let raw_host_pair = std::env::var("KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            missing_multi_host_decision(
                LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITES_MISSING_REASON_CODE,
            )
        })?;
    let (host_a, host_b) =
        parse_live_postgres_distributed_host_pair(&raw_host_pair).ok_or_else(|| {
            missing_multi_host_decision(
                LIVE_POSTGRES_MULTI_HOST_EXECUTION_HOST_PAIR_INVALID_REASON_CODE,
            )
        })?;
    Ok(format!("{host_a},{host_b}"))
}

fn distributed_topology_fingerprint() -> String {
    run_parallel_lane_topology_fingerprints(project_live_postgres_parallel_lane_topology_profiles())
        .into_iter()
        .find(|fingerprint| fingerprint.starts_with("distributed_label_parallel#"))
        .expect("distributed topology fingerprint should be present in multi-host projection")
}

fn topology_fingerprint_digest() -> String {
    let (_, digest) =
        project_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest(
            project_live_postgres_parallel_lane_topology_profiles(),
        );
    digest
}

pub(crate) fn resolve_live_postgres_multi_host_prerequisite_decision(
) -> LivePostgresMultiHostPrerequisiteDecision {
    let (gate_reason_code, maybe_database_url) = resolve_live_postgres_gate_decision();
    if gate_reason_code != LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE
        || maybe_database_url.is_none()
    {
        return missing_multi_host_decision(
            LIVE_POSTGRES_MULTI_HOST_EXECUTION_PREREQUISITES_MISSING_REASON_CODE,
        );
    }
    let host_pair_csv = match ready_host_pair_csv() {
        Ok(host_pair_csv) => host_pair_csv,
        Err(decision) => return decision,
    };
    LivePostgresMultiHostPrerequisiteDecision {
        reason_code: LIVE_POSTGRES_MULTI_HOST_EXECUTION_READY_REASON_CODE,
        reason_taxonomy_version: LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION,
        host_pair_csv: Some(host_pair_csv),
    }
}

pub(crate) fn project_live_postgres_multi_host_execution_bundle_selector_rows() -> Vec<String> {
    crate::live_postgres_multi_host_execution_bundle_selector_rows_for_test()
}

pub(crate) fn run_live_postgres_multi_host_execution_bundle_projection(
) -> Result<LivePostgresMultiHostExecutionProjection, LivePostgresMultiHostPrerequisiteDecision> {
    let prerequisite_decision = resolve_live_postgres_multi_host_prerequisite_decision();
    if prerequisite_decision.reason_code != LIVE_POSTGRES_MULTI_HOST_EXECUTION_READY_REASON_CODE {
        return Err(prerequisite_decision);
    }

    let host_pair_csv = prerequisite_decision
        .host_pair_csv
        .clone()
        .expect("ready multi-host prerequisite decision should include host pair csv");
    Ok(LivePostgresMultiHostExecutionProjection {
        reason_code: LIVE_POSTGRES_MULTI_HOST_EXECUTION_READY_REASON_CODE,
        reason_taxonomy_version: LIVE_POSTGRES_MULTI_HOST_EXECUTION_REASON_TAXONOMY_VERSION,
        host_pair_csv,
        distributed_topology_fingerprint: distributed_topology_fingerprint(),
        fingerprint_hash_order_normalization_digest_hex: topology_fingerprint_digest(),
    })
}
