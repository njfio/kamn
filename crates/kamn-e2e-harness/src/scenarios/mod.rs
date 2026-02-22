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
/// S-08 crash-recovery scenario module.
pub mod s08_crash_recovery;

/// Deterministic scenario definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioDefinition {
    /// Scenario ID marker.
    pub id: &'static str,
    /// Human-readable scenario name.
    pub name: &'static str,
    /// Priority marker.
    pub priority: &'static str,
}

/// Returns the core phase-3 scenario inventory.
pub fn core_scenarios() -> Vec<ScenarioDefinition> {
    vec![
        s01_discovery::definition(),
        s02_message::definition(),
        s03_group::definition(),
        s04_task::definition(),
        s05_escrow::definition(),
        s06_kolme_verify::definition(),
        s08_crash_recovery::definition(),
    ]
}
