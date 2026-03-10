use super::super::constants::*;
use super::fingerprint_support::*;
use super::topology_field_support::*;
pub(crate) fn extract_parallel_lane_topology_host_pair_id(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for host-pair extraction"
    );
    format!("{}->{}", fields[1], fields[2])
}

pub(crate) fn extract_parallel_lane_topology_host_pair_reverse_id(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for reverse host-pair extraction"
    );
    format!("{}->{}", fields[2], fields[1])
}

pub(crate) fn extract_parallel_lane_topology_id_host_pair_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id host-pair mapping extraction"
    );
    format!("{}->{}->{}", fields[0], fields[1], fields[2])
}

pub(crate) fn extract_parallel_lane_topology_id_lane_set_row(topology_fingerprint: &str) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id lane-set mapping extraction",
    );
    let lane_set = topology_lane_set(fields[0], "lane-set mapping");
    format!("{}->{}", fields[0], lane_set)
}

pub(crate) fn extract_parallel_lane_topology_id_lane_count_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology-id lane-count mapping extraction"
    );
    let lane_count = parse_parallel_lane_topology_bundle_fields(fields[3]).len();
    format!("{}->{}", fields[0], lane_count)
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode mapping extraction",
    );
    let host_mode = topology_host_mode(&fields);
    format!("{}->{}", fields[0], host_mode)
}

pub(crate) fn extract_parallel_lane_topology_id_host_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-cardinality mapping extraction",
    );
    let unique_host_cardinality = topology_unique_host_cardinality(&fields);
    format!("{}->{}", fields[0], unique_host_cardinality)
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-cardinality coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    let unique_host_cardinality = topology_unique_host_cardinality(&fields);
    format!("{}->{}->{}", fields[0], host_mode, unique_host_cardinality)
}

pub(crate) fn extract_parallel_lane_topology_id_host_pair_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-pair-cardinality coherence extraction",
    );
    let unique_host_cardinality = topology_unique_host_cardinality(&fields);
    format!(
        "{}->{}->{}->{}",
        fields[0], fields[1], fields[2], unique_host_cardinality
    )
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_host_pair_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    format!("{}->{}->{}->{}", fields[0], host_mode, fields[1], fields[2])
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_host_pair_cardinality_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-cardinality coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    let unique_host_cardinality = topology_unique_host_cardinality(&fields);
    format!(
        "{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], unique_host_cardinality
    )
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_count_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-count coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    let lane_set = topology_lane_set(
        fields[0],
        "host-mode-host-pair-lane-set-lane-count coherence extraction",
    );
    let lane_count = topology_lane_count(&fields);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_count
    )
}
