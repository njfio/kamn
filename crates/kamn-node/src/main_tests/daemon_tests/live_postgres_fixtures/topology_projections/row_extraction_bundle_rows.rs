use super::super::constants::*;
use super::super::models::*;
use super::fingerprint_support::*;
use super::topology_field_support::*;
pub(crate) fn extract_parallel_lane_topology_lane_id_bundle(topology_fingerprint: &str) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology lane-id-bundle extraction"
    );
    let mut lane_ids = parse_parallel_lane_topology_bundle_fields(fields[3])
        .iter()
        .map(|lane_fingerprint| {
            let lane_fields = parse_parallel_lane_fingerprint_fields(lane_fingerprint);
            assert_eq!(
                lane_fields.len(),
                LIVE_POSTGRES_PARALLEL_LANE_FINGERPRINT_FIELD_COUNT,
                "lane fingerprint should keep canonical field count for lane-id-bundle extraction"
            );
            lane_fields[0].to_owned()
        })
        .collect::<Vec<_>>();
    lane_ids.sort();
    lane_ids.join(&LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_ID_BUNDLE_DELIMITER.to_string())
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_id_bundle_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-id-bundle coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    let lane_set = topology_lane_set(
        fields[0],
        "host-mode-host-pair-lane-set-lane-id-bundle coherence extraction",
    );
    let lane_id_bundle = extract_parallel_lane_topology_lane_id_bundle(topology_fingerprint);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_id_bundle
    )
}

pub(crate) fn extract_parallel_lane_topology_lane_fingerprint_bundle(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_parallel_lane_topology_fingerprint_fields(topology_fingerprint);
    assert_eq!(
        fields.len(),
        LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_FINGERPRINT_FIELD_COUNT,
        "topology fingerprint should keep canonical field count for topology lane-fingerprint-bundle extraction"
    );
    let mut lane_fingerprints = parse_parallel_lane_topology_bundle_fields(fields[3])
        .iter()
        .map(|lane_fingerprint| (*lane_fingerprint).to_owned())
        .collect::<Vec<_>>();
    lane_fingerprints.sort();
    lane_fingerprints
        .join(&LIVE_POSTGRES_PARALLEL_LANE_TOPOLOGY_LANE_FINGERPRINT_BUNDLE_DELIMITER.to_string())
}

pub(crate) fn deterministic_fnv1a64_hex(input: &str) -> String {
    const FNV_OFFSET_BASIS_64: u64 = 0xcbf29ce484222325;
    const FNV_PRIME_64: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET_BASIS_64;
    for byte in input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME_64);
    }
    format!("{hash:016x}")
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_hash_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-fingerprint-hash coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    let lane_set = topology_lane_set(
        fields[0],
        "host-mode-host-pair-lane-set-lane-fingerprint-hash coherence extraction",
    );
    let lane_fingerprint_bundle =
        extract_parallel_lane_topology_lane_fingerprint_bundle(topology_fingerprint);
    let lane_fingerprint_hash = deterministic_fnv1a64_hex(&lane_fingerprint_bundle);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_fingerprint_hash
    )
}

pub(crate) fn extract_parallel_lane_topology_id_host_mode_host_pair_lane_set_lane_fingerprint_bundle_row(
    topology_fingerprint: &str,
) -> String {
    let fields = parse_topology_fields(
        topology_fingerprint,
        "topology fingerprint should keep canonical field count for topology-id host-mode-host-pair-lane-set-lane-fingerprint-bundle coherence extraction",
    );
    let host_mode = topology_host_mode(&fields);
    let lane_set = topology_lane_set(
        fields[0],
        "host-mode-host-pair-lane-set-lane-fingerprint-bundle coherence extraction",
    );
    let lane_fingerprint_bundle =
        extract_parallel_lane_topology_lane_fingerprint_bundle(topology_fingerprint);
    format!(
        "{}->{}->{}->{}->{}->{}",
        fields[0], host_mode, fields[1], fields[2], lane_set, lane_fingerprint_bundle
    )
}

pub(crate) fn permute_parallel_lane_topology_profiles(
    mut topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
    permutation_id: &str,
) -> Vec<LivePostgresParallelLaneTopologyProfile> {
    match permutation_id {
        "baseline" => topology_profiles,
        "reverse" => {
            topology_profiles.reverse();
            topology_profiles
        }
        "rotate_left_1" => {
            if !topology_profiles.is_empty() {
                topology_profiles.rotate_left(1);
            }
            topology_profiles
        }
        _ => panic!("unknown topology permutation: {permutation_id}"),
    }
}
