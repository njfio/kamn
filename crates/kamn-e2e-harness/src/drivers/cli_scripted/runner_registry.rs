use std::collections::BTreeMap;
use std::sync::Arc;

pub(super) type RunnerArc = Arc<super::LiveCliRunner>;
pub(super) type LiveRunnerMap = BTreeMap<&'static str, RunnerArc>;

const LIVE_SCENARIO_IDS: [&str; 15] = [
    "S-01", "S-02", "S-03", "S-04", "S-05", "S-06", "S-07", "S-08", "S-09", "S-10", "S-11", "S-12",
    "S-13", "S-14", "S-15",
];
const EARLY_SCENARIO_IDS: [&str; 5] = ["S-01", "S-02", "S-03", "S-04", "S-05"];
const LATE_SCENARIO_IDS: [&str; 10] = [
    "S-06", "S-07", "S-08", "S-09", "S-10", "S-11", "S-12", "S-13", "S-14", "S-15",
];

pub(super) fn shared_runner_map(live_runner: RunnerArc) -> LiveRunnerMap {
    LIVE_SCENARIO_IDS
        .into_iter()
        .map(|scenario_id| (scenario_id, Arc::clone(&live_runner)))
        .collect()
}

pub(super) fn explicit_runner_map(
    early_runners: [RunnerArc; 5],
    late_runners: [RunnerArc; 10],
) -> LiveRunnerMap {
    let mut live_runners = scenario_map(EARLY_SCENARIO_IDS, early_runners);
    live_runners.extend(scenario_map(LATE_SCENARIO_IDS, late_runners));
    live_runners
}

fn scenario_map<const N: usize>(
    scenario_ids: [&'static str; N],
    runners: [RunnerArc; N],
) -> LiveRunnerMap {
    scenario_ids.into_iter().zip(runners).collect()
}
