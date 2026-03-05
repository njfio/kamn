use super::*;

const OWNER_PHASE6_ALPHA: &str = "kamn:did:owner:phase6-alpha";
const OWNER_PHASE6_BETA: &str = "kamn:did:owner:phase6-beta";
const PARTITION_PRIMARY: u32 = 202401;
const PARTITION_SECONDARY: u32 = 202402;
const MESSAGE_A: &str = "message-a";
const MESSAGE_B: &str = "message-b";

pub(super) fn run_spec_c16_phase6_orchestration_tick_executes_retention_shred_projection_and_archive(
) {
    let owner_did = OWNER_PHASE6_ALPHA;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_PRIMARY, false))
        .expect("partition should register");

    let mut message_a = m8_message_input(owner_did, MESSAGE_A, 1_699_800_000);
    message_a.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(message_a)
        .expect("message-a should register");
    let mut message_b = m8_message_input(owner_did, MESSAGE_B, 1_699_810_000);
    message_b.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(message_b)
        .expect("message-b should register");

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(
                PARTITION_PRIMARY,
                vec![MESSAGE_B.to_owned(), MESSAGE_A.to_owned()],
            )]),
        ),
    )
    .expect("phase6 execution tick should succeed");

    assert_eq!(report.owner_did, owner_did);
    assert_eq!(report.due_candidate_count, 2);
    assert_eq!(
        report.shredded_message_ids,
        vec![MESSAGE_A.to_owned(), MESSAGE_B.to_owned()]
    );
    assert_eq!(report.projection_reports.len(), 1);
    assert!(report.projection_reports[0].all_messages_shredded);
    assert_eq!(report.archived_entries.len(), 1);
    assert_eq!(
        report.archived_entries[0].partition_name,
        "messages_2024_01"
    );
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE
    );
}

pub(super) fn run_spec_c17_phase6_orchestration_tick_orders_outputs_deterministically() {
    let owner_did = OWNER_PHASE6_BETA;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_PRIMARY, false))
        .expect("partition should register");
    m10_registry
        .register_partition(partition_input(PARTITION_SECONDARY, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("m-01", 1_699_700_000_u64),
        ("m-02", 1_699_700_100_u64),
        ("m-11", 1_699_700_200_u64),
        ("m-12", 1_699_700_300_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([
                (
                    PARTITION_SECONDARY,
                    vec!["m-12".to_owned(), "m-11".to_owned()],
                ),
                (
                    PARTITION_PRIMARY,
                    vec!["m-02".to_owned(), "m-01".to_owned()],
                ),
            ]),
        ),
    )
    .expect("phase6 execution tick should succeed");

    let projection_months: Vec<u32> = report
        .projection_reports
        .iter()
        .map(|report| report.partition_month_id)
        .collect();
    assert_eq!(
        projection_months,
        vec![PARTITION_PRIMARY, PARTITION_SECONDARY]
    );
    assert_eq!(
        report.shredded_message_ids,
        vec![
            "m-01".to_owned(),
            "m-02".to_owned(),
            "m-11".to_owned(),
            "m-12".to_owned(),
        ]
    );
    let archived_partition_names: Vec<String> = report
        .archived_entries
        .iter()
        .map(|entry| entry.partition_name.clone())
        .collect();
    assert_eq!(
        archived_partition_names,
        vec!["messages_2024_01".to_owned(), "messages_2024_02".to_owned()]
    );
}
