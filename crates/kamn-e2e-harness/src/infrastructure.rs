/// Infrastructure lifecycle phase marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfrastructurePhase {
    /// Infra startup.
    InfraUp,
    /// Agent deployment.
    AgentDeploy,
    /// Scenario execution.
    ScenarioRun,
    /// Evidence collection.
    Evidence,
    /// Teardown.
    Teardown,
}

/// Infrastructure lifecycle scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfrastructureLifecycle {
    /// Ordered lifecycle phases.
    pub phases: Vec<InfrastructurePhase>,
}

impl InfrastructureLifecycle {
    /// Returns deterministic default phase ordering.
    pub fn default_phases() -> Self {
        Self {
            phases: vec![
                InfrastructurePhase::InfraUp,
                InfrastructurePhase::AgentDeploy,
                InfrastructurePhase::ScenarioRun,
                InfrastructurePhase::Evidence,
                InfrastructurePhase::Teardown,
            ],
        }
    }
}
