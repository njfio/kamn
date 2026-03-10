#[path = "channel_task_probes/group_channel_probe.rs"]
mod group_channel_probe;
#[path = "channel_task_probes/task_lifecycle_probe.rs"]
mod task_lifecycle_probe;

pub(super) fn run_live_s03_cli_group_channel_probe() -> Result<(), String> {
    group_channel_probe::run_live_s03_cli_group_channel_probe()
}

pub(super) fn run_live_s04_cli_task_lifecycle_probe() -> Result<(), String> {
    task_lifecycle_probe::run_live_s04_cli_task_lifecycle_probe()
}
