#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresMatrixRow {
    pub(crate) scenario_id: &'static str,
    pub(crate) gate_reason_code: &'static str,
    pub(crate) daemon_phase6_reason_code: Option<&'static str>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresPhase6Projection {
    pub(crate) reason_code: String,
    pub(crate) reason_taxonomy_version: String,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresLoadProfile {
    pub(crate) profile_id: &'static str,
    pub(crate) args: Vec<String>,
    pub(crate) expected_reason_code: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresRolePairProfile {
    pub(crate) pair_id: &'static str,
    pub(crate) leg_a_profile_id: &'static str,
    pub(crate) leg_a_args: Vec<String>,
    pub(crate) leg_b_profile_id: &'static str,
    pub(crate) leg_b_args: Vec<String>,
    pub(crate) expected_reason_code: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresParallelLaneTopologyProfile {
    pub(crate) topology_id: &'static str,
    pub(crate) host_a: &'static str,
    pub(crate) host_b: &'static str,
    pub(crate) lanes: Vec<LivePostgresRolePairProfile>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresMultiHostPrerequisiteDecision {
    pub(crate) reason_code: &'static str,
    pub(crate) reason_taxonomy_version: &'static str,
    pub(crate) host_pair_csv: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LivePostgresMultiHostExecutionProjection {
    pub(crate) reason_code: &'static str,
    pub(crate) reason_taxonomy_version: &'static str,
    pub(crate) host_pair_csv: String,
    pub(crate) distributed_topology_fingerprint: String,
    pub(crate) fingerprint_hash_order_normalization_digest_hex: String,
}
