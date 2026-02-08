const DOC: &str = include_str!("../../../docs/foundation/message-lifecycle.md");

#[test]
fn doc_contains_lifecycle_chain_and_core_indexes() {
    assert!(DOC.contains("## Lifecycle State Machine"));
    assert!(DOC.contains(
        "Created -> Signed -> Broadcast -> Included -> Delivered -> Validated -> Rejected -> Expired"
    ));
    assert!(DOC.contains("ids_by_status(status)"));
    assert!(DOC.contains("ids_by_sender(sender)"));
    assert!(DOC.contains("ids_by_recipient(recipient)"));
}

#[test]
fn doc_contains_ttl_expiry_apis_and_rules() {
    assert!(DOC.contains("expire_message_if_overdue(message_id, observed_at)"));
    assert!(DOC.contains("expire_overdue_messages(observed_at)"));
    assert!(DOC.contains("observed_at > expires"));
    assert!(DOC.contains("`observed_at` passed to expiry APIs must be non-empty."));
}

#[test]
fn doc_contains_processor_proof_gated_validation_rules() {
    assert!(DOC.contains("## Processor Proof-Gated Validation"));
    assert!(DOC.contains("validate_with_processor_proof"));
    assert!(DOC.contains("Delivered -> Validated"));
}

#[test]
fn doc_contains_snapshot_persistence_and_restore_contract_rules() {
    assert!(DOC.contains("## Snapshot Persistence and Restore Contract Rules"));
    assert!(DOC.contains("export_snapshot()"));
    assert!(DOC.contains("restore_snapshot(snapshot)"));
    assert!(DOC.contains("MessageLifecycleSnapshotStore"));
    assert!(DOC.contains("recover_latest_and_repair()"));
    assert!(DOC.contains("MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION"));
}

#[test]
fn regression_requires_active_record_ttl_expiry_rule() {
    // Regression: #563
    assert!(DOC.contains("Expiry APIs only transition active records"));
    assert!(DOC.contains("`Included`, `Delivered`) to `Expired`."));
}

#[test]
fn regression_requires_tampered_proof_transition_block_rule() {
    // Regression: #510
    assert!(DOC.contains("tampered proof artifacts must not advance message state"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane_commands() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core message_lifecycle::tests::"));
    assert!(DOC.contains("cargo test -p kamn-core --test message_lifecycle_queries"));
    assert!(DOC.contains("cargo test -p kamn-core --test message_lifecycle_proof_admission"));
    assert!(DOC.contains("bash scripts/message/run_message_lifecycle_contract_lane.sh"));
    assert!(
        DOC.contains(
            "cargo test -p kamn-core message_lifecycle::tests::performance_message_lifecycle_snapshot_deep_lane_stress -- --ignored"
        )
    );
    assert!(DOC.contains("bash scripts/message/run_message_lifecycle_deep_lane.sh"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_snapshot_restore_guard_rules() {
    // Regression: #617
    assert!(DOC.contains("duplicate message IDs on restore are rejected (`Regression: #617`)"));
    assert!(DOC.contains("status/history mismatch on restore is rejected (`Regression: #617`)"));
}
