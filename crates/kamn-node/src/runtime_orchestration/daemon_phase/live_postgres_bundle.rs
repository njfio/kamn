use std::collections::BTreeSet;

const DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SCHEMA_VERSION: &str =
    "kamn.runtime.daemon.phase6-live-postgres.multi-host-execution-bundle.v1";
const DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX: &str =
    "main_tests::daemon_tests::";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_DUPLICATE_ROWS_REASON_CODE: &str =
    "live_postgres_selector_bundle_duplicate_rows";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_PREFIX_VIOLATION_REASON_CODE: &str =
    "live_postgres_selector_bundle_prefix_violation";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_FORMAT_VIOLATION_REASON_CODE: &str =
    "live_postgres_selector_bundle_row_format_violation";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_ID_VIOLATION_REASON_CODE: &str =
    "live_postgres_selector_bundle_row_id_violation";
const DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_COUNT_MISMATCH_REASON_CODE: &str =
    "live_postgres_selector_bundle_row_count_mismatch";
const DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS: [(&str, &str); 6] = [
    (
        "b01_runtime_matrix_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs",
    ),
    (
        "b02_parallel_lane_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable",
    ),
    (
        "b03_topology_mapping_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable",
    ),
    (
        "b04_topology_coherence_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_is_stable",
    ),
    (
        "b05_fingerprint_stability_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable",
    ),
    (
        "b06_multi_host_execution_bundle",
        "integration_runtime_daemon_phase6_live_postgres_validation_slice_multi_host_execution_bundle_is_stable",
    ),
];

pub(super) fn live_postgres_schema_version() -> &'static str {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SCHEMA_VERSION
}

pub(super) fn live_postgres_selector_prefix() -> &'static str {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX
}

pub(super) fn project_live_postgres_multi_host_execution_bundle_selector_rows() -> Vec<String> {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS
        .iter()
        .map(|(row_id, row_suffix)| {
            format!(
                "{row_id}->{DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX}{row_suffix}"
            )
        })
        .collect()
}

pub(super) fn daemon_live_postgres_multi_host_execution_bundle_row_count() -> usize {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS.len()
}

pub(super) fn project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(
    rows: &[String],
) -> String {
    deterministic_fnv1a64_hex(&rows.join(","))
}

pub(super) fn validate_live_postgres_selector_bundle(
    rows: &[String],
    expected_row_count: usize,
) -> Result<(), &'static str> {
    if rows.len() != expected_row_count {
        return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_COUNT_MISMATCH_REASON_CODE);
    }
    let canonical_row_ids = daemon_live_postgres_multi_host_execution_bundle_row_ids();
    let mut dedupe = BTreeSet::new();
    for row in rows {
        if !dedupe.insert(row.as_str()) {
            return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_DUPLICATE_ROWS_REASON_CODE);
        }
        validate_live_postgres_selector_row(row, &canonical_row_ids)?;
    }
    Ok(())
}

fn daemon_live_postgres_multi_host_execution_bundle_row_ids() -> BTreeSet<&'static str> {
    DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_ROWS
        .iter()
        .map(|(row_id, _)| *row_id)
        .collect()
}

fn validate_live_postgres_selector_row(
    row: &str,
    canonical_row_ids: &BTreeSet<&'static str>,
) -> Result<(), &'static str> {
    let Some((row_id, selector)) = row.split_once("->") else {
        return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_FORMAT_VIOLATION_REASON_CODE);
    };
    if row_id.is_empty() || selector.is_empty() || selector.contains("->") {
        return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_FORMAT_VIOLATION_REASON_CODE);
    }
    if !canonical_row_ids.contains(row_id) {
        return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_ROW_ID_VIOLATION_REASON_CODE);
    }
    if !selector.starts_with(DAEMON_LIVE_POSTGRES_MULTI_HOST_EXECUTION_BUNDLE_SELECTOR_PREFIX) {
        return Err(DAEMON_LIVE_POSTGRES_SELECTOR_BUNDLE_PREFIX_VIOLATION_REASON_CODE);
    }
    Ok(())
}

fn deterministic_fnv1a64_hex(input: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
pub(crate) fn live_postgres_multi_host_execution_bundle_selector_rows_for_test() -> Vec<String> {
    project_live_postgres_multi_host_execution_bundle_selector_rows()
}

#[cfg(test)]
pub(crate) fn live_postgres_multi_host_execution_bundle_row_count_for_test() -> usize {
    daemon_live_postgres_multi_host_execution_bundle_row_count()
}

#[cfg(test)]
pub(crate) fn live_postgres_multi_host_execution_bundle_selector_rows_fingerprint_for_test(
) -> String {
    let rows = project_live_postgres_multi_host_execution_bundle_selector_rows();
    project_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint(rows.as_slice())
}

#[cfg(test)]
pub(crate) fn validate_live_postgres_selector_bundle_for_test(
    rows: &[String],
    expected_row_count: usize,
) -> Result<(), &'static str> {
    validate_live_postgres_selector_bundle(rows, expected_row_count)
}
