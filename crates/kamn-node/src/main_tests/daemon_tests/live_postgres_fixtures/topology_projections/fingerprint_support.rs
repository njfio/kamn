use super::super::constants::*;
use super::super::models::*;
pub(crate) fn format_parallel_lane_fingerprint(
    lane_id: &str,
    leg_a_projection: &LivePostgresPhase6Projection,
    leg_b_projection: &LivePostgresPhase6Projection,
) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        lane_id,
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_a_projection.reason_code.as_str(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_a_projection.reason_taxonomy_version.as_str(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_b_projection.reason_code.as_str(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER,
        leg_b_projection.reason_taxonomy_version.as_str()
    )
}

pub(crate) fn parse_parallel_lane_fingerprint_fields(fingerprint: &str) -> Vec<&str> {
    fingerprint
        .split(LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_DELIMITER)
        .collect::<Vec<_>>()
}

pub(crate) fn assert_parallel_lane_fingerprint_schema(
    fingerprint: &str,
    expected_lane_ids: &[&str],
) {
    let fields = parse_parallel_lane_fingerprint_fields(fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT,
        "fingerprint should contain the canonical number of schema fields"
    );
    assert!(
        expected_lane_ids
            .iter()
            .any(|lane_id| *lane_id == fields[0]),
        "fingerprint lane id {} should be one of {:?}",
        fields[0],
        expected_lane_ids
    );
    assert_parallel_lane_reason_codes(&fields);
    assert_parallel_lane_taxonomy_versions(&fields);
}

fn assert_parallel_lane_reason_codes(fields: &[&str]) {
    assert!(
        [
            LIVE_POSTGRES_MATRIX_PHASE6_APPLIED_REASON_CODE,
            LIVE_POSTGRES_MATRIX_PHASE6_DEFERRED_REASON_CODE,
        ]
        .contains(&fields[1]),
        "fingerprint leg A reason should remain in the canonical reason taxonomy set"
    );
    assert_eq!(
        fields[1], fields[3],
        "parallel lane fingerprint should keep leg A and leg B reason codes aligned"
    );
}

fn assert_parallel_lane_taxonomy_versions(fields: &[&str]) {
    assert_eq!(
        fields[2], LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "fingerprint leg A taxonomy should remain canonical"
    );
    assert_eq!(
        fields[4], LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "fingerprint leg B taxonomy should remain canonical"
    );
}

pub(crate) fn format_parallel_lane_topology_fingerprint(
    topology_id: &str,
    host_a: &str,
    host_b: &str,
    lane_fingerprints: Vec<String>,
) -> String {
    format!(
        "{}{}{}{}{}{}{}",
        topology_id,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        host_a,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        host_b,
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER,
        lane_fingerprints
            .join(&LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER.to_string())
    )
}

pub(crate) fn parse_parallel_lane_topology_fingerprint_fields(fingerprint: &str) -> Vec<&str> {
    fingerprint
        .split(LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_DELIMITER)
        .collect::<Vec<_>>()
}

pub(crate) fn parse_parallel_lane_topology_bundle_fields(bundle: &str) -> Vec<&str> {
    if bundle.is_empty() {
        Vec::new()
    } else {
        bundle
            .split(LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_BUNDLE_DELIMITER)
            .collect::<Vec<_>>()
    }
}
