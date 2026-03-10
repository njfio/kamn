use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) type RunnerArc = Arc<super::LiveMcpProbe>;
pub(super) type LiveRunnerMap = BTreeMap<&'static str, RunnerArc>;

const LIVE_SCENARIO_IDS: [&str; 15] = [
    "S-01", "S-02", "S-03", "S-04", "S-05", "S-06", "S-07", "S-08", "S-09", "S-10", "S-11", "S-12",
    "S-13", "S-14", "S-15",
];

pub(super) fn shared_runner_map(runner: RunnerArc) -> LiveRunnerMap {
    scenario_map(LIVE_SCENARIO_IDS.map(|_| runner.clone()))
}

pub(super) fn explicit_runner_map(runners: [RunnerArc; 15]) -> LiveRunnerMap {
    scenario_map(runners)
}

fn scenario_map(runners: [RunnerArc; 15]) -> LiveRunnerMap {
    LIVE_SCENARIO_IDS.into_iter().zip(runners).collect()
}
