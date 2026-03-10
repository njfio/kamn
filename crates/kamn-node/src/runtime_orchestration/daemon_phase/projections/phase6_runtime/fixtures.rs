use super::super::super::super::*;

type Phase6Registries = (
    kamn_core::DataLayerM8ComplianceRegistry,
    kamn_core::DataLayerM10PartitionLifecycleRegistry,
    std::collections::BTreeMap<u32, Vec<String>>,
);

pub(super) fn build_phase6_registries(
    owner_did: &str,
    has_shutdown_signal: bool,
) -> Result<Phase6Registries, ConfigError> {
    let mut m8_registry = kamn_core::DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = kamn_core::DataLayerM10PartitionLifecycleRegistry::new();
    let mut partition_message_ids_by_month = std::collections::BTreeMap::new();
    if has_shutdown_signal {
        register_deferred_phase6_fixture(
            owner_did,
            &mut m8_registry,
            &mut m10_registry,
            &mut partition_message_ids_by_month,
        )?;
    } else {
        register_executed_phase6_fixture(
            owner_did,
            &mut m8_registry,
            &mut m10_registry,
            &mut partition_message_ids_by_month,
        )?;
    }
    Ok((m8_registry, m10_registry, partition_message_ids_by_month))
}

fn register_executed_phase6_fixture(
    owner_did: &str,
    m8_registry: &mut kamn_core::DataLayerM8ComplianceRegistry,
    m10_registry: &mut kamn_core::DataLayerM10PartitionLifecycleRegistry,
    partition_message_ids_by_month: &mut std::collections::BTreeMap<u32, Vec<String>>,
) -> Result<(), ConfigError> {
    register_partition(m10_registry, 202401)?;
    for (message_id, created_at_epoch_seconds) in executed_phase6_messages() {
        register_phase6_message(owner_did, m8_registry, message_id, created_at_epoch_seconds)?;
    }
    partition_message_ids_by_month.insert(202401, executed_phase6_ids());
    Ok(())
}

fn executed_phase6_messages() -> [(&'static str, u64); 2] {
    [
        ("daemon-phase6-message-a", 1_699_700_000_u64),
        ("daemon-phase6-message-b", 1_699_700_100_u64),
    ]
}

fn executed_phase6_ids() -> Vec<String> {
    vec![
        "daemon-phase6-message-b".to_owned(),
        "daemon-phase6-message-a".to_owned(),
    ]
}

fn register_deferred_phase6_fixture(
    owner_did: &str,
    m8_registry: &mut kamn_core::DataLayerM8ComplianceRegistry,
    m10_registry: &mut kamn_core::DataLayerM10PartitionLifecycleRegistry,
    partition_message_ids_by_month: &mut std::collections::BTreeMap<u32, Vec<String>>,
) -> Result<(), ConfigError> {
    register_partition(m10_registry, 202601)?;
    register_phase6_message(
        owner_did,
        m8_registry,
        "daemon-phase6-deferred-message",
        1_699_999_990,
    )?;
    partition_message_ids_by_month
        .insert(202601, vec!["daemon-phase6-deferred-message".to_owned()]);
    Ok(())
}

fn register_partition(
    m10_registry: &mut kamn_core::DataLayerM10PartitionLifecycleRegistry,
    partition_month_id: u32,
) -> Result<(), ConfigError> {
    m10_registry
        .register_partition(kamn_core::DataLayerM10PartitionRecordInput {
            partition_month_id,
            all_messages_shredded: false,
        })
        .map(|_| ())
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))
}

fn register_phase6_message(
    owner_did: &str,
    m8_registry: &mut kamn_core::DataLayerM8ComplianceRegistry,
    message_id: &str,
    created_at_epoch_seconds: u64,
) -> Result<(), ConfigError> {
    m8_registry
        .register_message(kamn_core::DataLayerM8MessageRecordInput {
            owner_did: owner_did.to_owned(),
            message_id: message_id.to_owned(),
            created_at_epoch_seconds,
            content_hash: format!("hash:{message_id}"),
            hash_chain_prev: format!("prev:{message_id}"),
            retention_class: kamn_core::DataLayerM8RetentionClass::Ephemeral,
            retention_extension_seconds: 0,
            wrapped_keys: vec![kamn_core::DataLayerM8WrappedCekInput {
                recipient_did: "kamn:did:agent:daemon-phase6".to_owned(),
                wrapped_cek: format!("cek:{message_id}"),
            }],
        })
        .map(|_| ())
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))
}
