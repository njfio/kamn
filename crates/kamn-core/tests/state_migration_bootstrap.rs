use kamn_core::{
    bootstrap, bootstrap_from_state_version, canonical_state_key, ConfigError, MigrationRegistry,
    MigrationStep, NodeConfig, NodeRole, StateVersion, SyncMode, APP_STATE_VERSION,
};

fn sample_config() -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role: NodeRole::Processor,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: true,
        sync_mode: SyncMode::Fast,
    }
}

#[test]
fn functional_bootstrap_uses_current_schema_for_fresh_state() {
    let plan = bootstrap(sample_config()).expect("bootstrap should succeed");

    assert_eq!(plan.state_schema.version, APP_STATE_VERSION);
    assert_eq!(plan.migration_plan.from, APP_STATE_VERSION);
    assert_eq!(plan.migration_plan.to, APP_STATE_VERSION);
    assert!(plan.migration_plan.steps.is_empty());
    assert_eq!(plan.namespaces, plan.state_schema.namespaces);
}

#[test]
fn integration_registry_plan_and_key_serialization_work_together() {
    let mut registry = MigrationRegistry::new();
    registry
        .register(MigrationStep::new(
            "tasks-v0-v1",
            StateVersion(0),
            APP_STATE_VERSION,
            "Initialize task namespace layout",
            &["kamn.tasks.state"],
        ))
        .expect("registry step should be valid");

    let plan = registry
        .build_plan(StateVersion(0), APP_STATE_VERSION)
        .expect("upgrade plan should build");

    assert_eq!(plan.steps.len(), 1);
    let key = canonical_state_key(plan.steps[0].namespaces[0], "migration", "tasks_v0_v1")
        .expect("migration key should serialize");
    assert_eq!(key, "kamn.tasks.state:migration:tasks_v0_v1");
}

#[test]
fn regression_bootstrap_rejects_state_downgrade_attempt() {
    // Regression: #17
    let result =
        bootstrap_from_state_version(sample_config(), StateVersion(APP_STATE_VERSION.0 + 1));
    match result {
        Err(ConfigError::MigrationPlan(message)) => {
            assert!(message.contains("invalid migration plan range"));
        }
        other => panic!("expected migration planning failure, got {other:?}"),
    }
}
