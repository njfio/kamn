use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    evaluate_cross_store_replay_consistency, ChannelSnapshot, ChannelStore,
    MessageLifecycleSnapshot, MessageLifecycleStore, RuntimeSnapshot, TaskOperationEngine,
    TaskOperationSnapshot,
};

fn build_channel_snapshot() -> Result<ChannelSnapshot, String> {
    let mut store = ChannelStore::new();
    store
        .create_direct(
            "channel-alpha",
            "kamn:did:agent:sender-a",
            "kamn:did:agent:recipient-a",
        )
        .map_err(|error| format!("channel snapshot setup failed: {error}"))?;
    Ok(store.export_snapshot())
}

fn build_message_snapshot() -> Result<MessageLifecycleSnapshot, String> {
    let mut store = MessageLifecycleStore::new();
    store
        .register(
            "message-alpha",
            "kamn:did:agent:sender-a",
            vec!["kamn:did:agent:recipient-a".to_owned()],
            "2026-02-20T00:00:00Z",
            "2026-02-20T00:10:00Z",
        )
        .map_err(|error| format!("message snapshot setup failed: {error}"))?;
    Ok(store.export_snapshot())
}

fn build_task_snapshot() -> Result<TaskOperationSnapshot, String> {
    let mut engine = TaskOperationEngine::new();
    engine
        .submit(
            "task-alpha",
            "kamn:did:agent:requester-a",
            "replay consistency checker fixture task",
        )
        .map_err(|error| format!("task snapshot setup failed: {error}"))?;
    Ok(engine.export_snapshot())
}

fn run() -> Result<(), String> {
    let runtime_snapshot = RuntimeSnapshot::with_cursor(6, "state-6", 6)
        .map_err(|error| format!("runtime snapshot setup failed: {error}"))?;
    let channel_snapshot = build_channel_snapshot()?;
    let message_snapshot = build_message_snapshot()?;
    let task_snapshot = build_task_snapshot()?;

    let report = evaluate_cross_store_replay_consistency(
        Some(runtime_snapshot),
        Some(channel_snapshot),
        Some(message_snapshot),
        Some(task_snapshot),
    );

    if report.policy_status_marker() != "verified" {
        return Err(format!(
            "cross-store replay consistency policy marker drifted: status={:?} marker={}",
            report.status(),
            report.policy_status_marker()
        ));
    }
    if report.status().policy_status_marker() != report.policy_status_marker() {
        return Err(format!(
            "cross-store replay consistency policy marker mismatch: status={:?} marker={}",
            report.status(),
            report.policy_status_marker()
        ));
    }
    if report.status().policy_status_marker() == "violated" {
        return Err(format!(
            "cross-store replay consistency failed unexpectedly: marker={} reason={}",
            report.policy_status_marker(),
            report.reason_code()
        ));
    }
    if report.reason_code() != "none" {
        return Err(format!(
            "cross-store replay consistency reason drifted: {}",
            report.reason_code()
        ));
    }

    println!(
        "cross_store_replay_consistency_policy_status={}",
        report.policy_status_marker()
    );
    println!("cross_store_replay_consistency_contract_lane_status=verified");
    println!(
        "cross_store_replay_reason_taxonomy_version={}",
        cross_store_replay_reason_taxonomy_version()
    );
    println!(
        "cross_store_replay_reason_codes_csv={}",
        cross_store_replay_reason_codes_csv()
    );
    println!("cross-store replay consistency contract lane tests passed.");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
