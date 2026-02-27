use std::fs;

fn read_core_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full = format!("{root}/{path}");
    fs::read_to_string(full).unwrap_or_default()
}

#[test]
fn core_split_phase1_contract_declares_snapshot_journal_dependency() {
    let cargo_toml = read_core_file("Cargo.toml");
    assert!(
        cargo_toml.contains("kamn-snapshot-journal"),
        "kamn-core Cargo.toml should depend on extracted kamn-snapshot-journal crate"
    );
}

#[test]
fn core_split_phase1_contract_uses_snapshot_journal_crate_in_snapshot_domains() {
    let channel_models = read_core_file("src/channel_models.rs");
    let task_operations = read_core_file("src/task_operations.rs");
    let message_lifecycle = read_core_file("src/message_lifecycle.rs");

    assert!(
        channel_models.contains("kamn_snapshot_journal"),
        "channel_models should consume extracted snapshot journal crate"
    );
    assert!(
        task_operations.contains("kamn_snapshot_journal"),
        "task_operations should consume extracted snapshot journal crate"
    );
    assert!(
        message_lifecycle.contains("kamn_snapshot_journal"),
        "message_lifecycle should consume extracted snapshot journal crate"
    );
}
