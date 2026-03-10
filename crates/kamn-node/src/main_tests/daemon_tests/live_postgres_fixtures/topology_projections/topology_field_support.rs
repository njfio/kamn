use super::super::constants::*;
use super::fingerprint_support::*;

pub(crate) fn parse_topology_fields<'a>(
    topology_fingerprint: &'a str,
    context: &str,
) -> Vec<&'a str> {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "{context}"
    );
    fields
}

pub(crate) fn topology_host_mode(fields: &[&str]) -> &'static str {
    if fields[1] == fields[2] {
        "same_host"
    } else {
        "distributed_label"
    }
}

pub(crate) fn topology_lane_set(topology_id: &str, context: &str) -> &'static str {
    match topology_id {
        "same_host_parallel" => "symmetric_parallel",
        "distributed_label_parallel" => "asymmetric_parallel",
        _ => panic!("unknown topology id {topology_id} for {context}"),
    }
}

pub(crate) fn topology_unique_host_cardinality(fields: &[&str]) -> usize {
    if fields[1] == fields[2] {
        1
    } else {
        2
    }
}

pub(crate) fn topology_lane_count(fields: &[&str]) -> usize {
    parse_parallel_lane_topology_bundle_fields(fields[3]).len()
}
