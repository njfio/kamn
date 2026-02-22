/// S-01 discovery scenario module.
pub mod s01_discovery;
/// S-02 direct-message scenario module.
pub mod s02_message;
/// S-03 group scenario module.
pub mod s03_group;
/// S-04 task lifecycle scenario module.
pub mod s04_task;
/// S-05 escrow scenario module.
pub mod s05_escrow;
/// S-06 proof verification scenario module.
pub mod s06_kolme_verify;
/// S-07 replay-protection scenario module.
pub mod s07_replay_protection;
/// S-08 crash-recovery scenario module.
pub mod s08_crash_recovery;
/// S-09 transport-failover scenario module.
pub mod s09_transport_failover;
/// S-10 topology-coherence scenario module.
pub mod s10_topology_coherence;
/// S-11 signer-rotation scenario module.
pub mod s11_signer_rotation;
/// S-12 retention-deletion scenario module.
pub mod s12_retention_deletion;
/// S-13 bridge-forwarding scenario module.
pub mod s13_bridge_forwarding;
/// S-14 batch-merkle scenario module.
pub mod s14_batch_merkle;
/// S-15 performance-smoke scenario module.
pub mod s15_performance_smoke;

/// Deterministic scenario definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDefinition {
    /// Scenario ID marker.
    pub id: &'static str,
    /// Human-readable scenario name.
    pub name: &'static str,
    /// Priority marker.
    pub priority: &'static str,
    /// Ordered scenario steps.
    pub steps: &'static [&'static str],
    /// Scenario verifiable output artifacts.
    pub verifiable_outputs: &'static [&'static str],
    /// Scenario pass criteria.
    pub pass_criteria: &'static [&'static str],
}

/// Returns the full PRD scenario matrix (`S-01` through `S-15`).
pub fn all_scenarios() -> Vec<ScenarioDefinition> {
    vec![
        s01_discovery::definition(),
        s02_message::definition(),
        s03_group::definition(),
        s04_task::definition(),
        s05_escrow::definition(),
        s06_kolme_verify::definition(),
        s07_replay_protection::definition(),
        s08_crash_recovery::definition(),
        s09_transport_failover::definition(),
        s10_topology_coherence::definition(),
        s11_signer_rotation::definition(),
        s12_retention_deletion::definition(),
        s13_bridge_forwarding::definition(),
        s14_batch_merkle::definition(),
        s15_performance_smoke::definition(),
    ]
}

/// Returns the harness default scenario inventory.
pub fn core_scenarios() -> Vec<ScenarioDefinition> {
    all_scenarios()
}
