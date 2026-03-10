use super::super::constants::*;
use super::super::matrix_profiles::*;
use super::super::models::*;
use super::fingerprint_support::*;
fn assert_parallel_lane_leg_projection(
    projection: &LivePostgresPhase6Projection,
    expected_reason_code: &str,
    pair_id: &str,
    leg_profile_id: &str,
    leg_label: &str,
) {
    assert_eq!(
        projection.reason_code, expected_reason_code,
        "lane {pair_id} {leg_label} ({leg_profile_id}) should project expected reason code"
    );
    assert_eq!(
        projection.reason_taxonomy_version, LIVE_POSTGRES_DAEMON_REASON_TAXONOMY_VERSION,
        "lane {pair_id} {leg_label} taxonomy should remain stable"
    );
}

fn run_parallel_lane_fingerprint(lane: LivePostgresRolePairProfile) -> String {
    let (leg_a_projection, leg_b_projection) =
        run_parallel_phase6_projections(lane.leg_a_args, lane.leg_b_args);
    assert_parallel_lane_leg_projection(
        &leg_a_projection,
        lane.expected_reason_code,
        lane.pair_id,
        lane.leg_a_profile_id,
        "leg A",
    );
    assert_parallel_lane_leg_projection(
        &leg_b_projection,
        lane.expected_reason_code,
        lane.pair_id,
        lane.leg_b_profile_id,
        "leg B",
    );
    format_parallel_lane_fingerprint(lane.pair_id, &leg_a_projection, &leg_b_projection)
}

pub(crate) fn run_parallel_lane_set_fingerprints(
    lanes: Vec<LivePostgresRolePairProfile>,
) -> Vec<String> {
    let mut fingerprints = Vec::with_capacity(lanes.len());
    for lane in lanes {
        fingerprints.push(run_parallel_lane_fingerprint(lane));
    }
    fingerprints.sort();
    fingerprints
}

pub(crate) fn run_parallel_lane_topology_fingerprints(
    topology_profiles: Vec<LivePostgresParallelLaneTopologyProfile>,
) -> Vec<String> {
    let mut topology_fingerprints = Vec::with_capacity(topology_profiles.len());
    for topology_profile in topology_profiles {
        let expected_lane_ids = topology_profile
            .lanes
            .iter()
            .map(|lane| lane.pair_id)
            .collect::<Vec<_>>();
        let lane_fingerprints = run_parallel_lane_set_fingerprints(topology_profile.lanes);
        for lane_fingerprint in &lane_fingerprints {
            assert_parallel_lane_fingerprint_schema(lane_fingerprint, &expected_lane_ids);
        }
        topology_fingerprints.push(format_parallel_lane_topology_fingerprint(
            topology_profile.topology_id,
            topology_profile.host_a,
            topology_profile.host_b,
            lane_fingerprints,
        ));
    }
    topology_fingerprints.sort();
    topology_fingerprints
}
