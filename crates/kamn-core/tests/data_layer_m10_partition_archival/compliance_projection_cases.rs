use super::*;

const OWNER_ALPHA: &str = "kamn:did:owner:alpha";
const OWNER_BETA: &str = "kamn:did:owner:beta";
const PARTITION_MONTH_PRIMARY: u32 = 202401;
const NOW_MONTH_ID: u32 = 202602;
const ARCHIVE_PREFIX: &str = "s3://kamn-archive/messages";
const MESSAGE_ONE: &str = "m10-m8-msg-1";
const MESSAGE_TWO: &str = "m10-m8-msg-2";

pub(super) fn run_spec_c06_partition_shred_completeness_can_be_projected_from_m8_lifecycle_records(
) {
    let owner_did = OWNER_ALPHA;
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_MONTH_PRIMARY, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_ONE, 1_708_560_100))
        .expect("message one should register");
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_TWO, 1_708_560_110))
        .expect("message two should register");

    let initial_projection: DataLayerM10ComplianceShredProjectionReport = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, PARTITION_MONTH_PRIMARY, vec![MESSAGE_ONE, MESSAGE_TWO]),
        )
        .expect("initial projection should succeed");
    assert_eq!(
        initial_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
    );
    assert!(!initial_projection.all_messages_shredded);
    assert_eq!(initial_projection.shredded_partition_messages, 0);
    assert_eq!(
        initial_projection.projection_reason_code,
        DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE
    );

    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_ONE.to_owned(),
            shredded_at_epoch_seconds: 1_708_560_200,
        })
        .expect("first message should shred");
    let mid_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, PARTITION_MONTH_PRIMARY, vec![MESSAGE_ONE, MESSAGE_TWO]),
        )
        .expect("mid projection should succeed");
    assert_eq!(
        mid_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_FALSE_REASON_CODE
    );
    assert_eq!(mid_projection.shredded_partition_messages, 1);

    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_TWO.to_owned(),
            shredded_at_epoch_seconds: 1_708_560_210,
        })
        .expect("second message should shred");
    let final_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, PARTITION_MONTH_PRIMARY, vec![MESSAGE_ONE, MESSAGE_TWO]),
        )
        .expect("final projection should succeed");
    assert_eq!(
        final_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
    );
    assert!(final_projection.all_messages_shredded);
    assert_eq!(final_projection.shredded_partition_messages, 2);

    let archived = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: NOW_MONTH_ID,
            active_retention_months: 1,
            object_storage_prefix: ARCHIVE_PREFIX.to_owned(),
        })
        .expect("archive due should succeed");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
}

pub(super) fn run_spec_c07_partition_shred_projection_fails_closed_when_m8_message_lookup_is_missing(
) {
    let owner_did = OWNER_ALPHA;
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_MONTH_PRIMARY, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(
            owner_did,
            "m10-m8-msg-present",
            1_708_560_100,
        ))
        .expect("message should register");

    let missing = m10_registry.project_partition_shred_completeness_from_m8(
        &m8_registry,
        project_request(
            owner_did,
            PARTITION_MONTH_PRIMARY,
            vec!["m10-m8-msg-present", "m10-m8-msg-missing"],
        ),
    );
    assert!(matches!(
        missing,
        Err(
            DataLayerM10PartitionLifecycleError::ComplianceProjectionFailed {
                reason_code: DATA_LAYER_M10_COMPLIANCE_LOOKUP_FAILED_REASON_CODE,
                ..
            }
        )
    ));
}

pub(super) fn run_spec_c08_partition_projection_accepts_canonical_equivalent_owner_dids() {
    let owner_did = OWNER_ALPHA;
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_MONTH_PRIMARY, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_ONE, 1_708_560_100))
        .expect("message should register");

    let projection = m10_registry.project_partition_shred_completeness_from_m8(
        &m8_registry,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: "  kamn:did:owner:alpha  ".to_owned(),
            owner_did: owner_did.to_owned(),
            partition_month_id: PARTITION_MONTH_PRIMARY,
            partition_message_ids: vec![MESSAGE_ONE.to_owned()],
        },
    );
    assert!(
        projection.is_ok(),
        "canonical-equivalent owner DIDs should authorize projection scope"
    );
}

pub(super) fn run_spec_c09_partition_projection_denies_non_equivalent_owner_dids() {
    let owner_did = OWNER_ALPHA;
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_MONTH_PRIMARY, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_ONE, 1_708_560_100))
        .expect("message should register");

    let denied = m10_registry.project_partition_shred_completeness_from_m8(
        &m8_registry,
        DataLayerM10ComplianceShredProjectionRequest {
            requester_owner_did: OWNER_BETA.to_owned(),
            owner_did: owner_did.to_owned(),
            partition_month_id: PARTITION_MONTH_PRIMARY,
            partition_message_ids: vec![MESSAGE_ONE.to_owned()],
        },
    );
    assert!(matches!(
        denied,
        Err(DataLayerM10PartitionLifecycleError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M10_COMPLIANCE_OWNER_SCOPE_DENIED_REASON_CODE,
        })
    ));
}

pub(super) fn run_spec_c10_partition_projection_marks_legal_hold_as_archival_denied_reason() {
    let owner_did = OWNER_ALPHA;
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_MONTH_PRIMARY, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_ONE, 1_708_560_100))
        .expect("message one should register");
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_TWO, 1_708_560_110))
        .expect("message two should register");

    m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_TWO.to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should apply");
    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_ONE.to_owned(),
            shredded_at_epoch_seconds: 1_708_560_200,
        })
        .expect("first message should shred");

    let projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, PARTITION_MONTH_PRIMARY, vec![MESSAGE_ONE, MESSAGE_TWO]),
        )
        .expect("projection should succeed");
    assert_eq!(
        projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
    );
    assert!(!projection.all_messages_shredded);
    assert_eq!(projection.shredded_partition_messages, 1);
    assert_eq!(
        projection.projection_reason_code,
        DATA_LAYER_M10_COMPLIANCE_PROJECTION_APPLIED_REASON_CODE
    );
}

pub(super) fn run_spec_c11_partition_archival_remains_blocked_until_legal_hold_is_released_and_shred_completes(
) {
    let owner_did = OWNER_ALPHA;
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(PARTITION_MONTH_PRIMARY, false))
        .expect("partition should register");

    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_ONE, 1_708_560_100))
        .expect("message one should register");
    m8_registry
        .register_message(m8_message_input(owner_did, MESSAGE_TWO, 1_708_560_110))
        .expect("message two should register");

    m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_TWO.to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should apply");
    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_ONE.to_owned(),
            shredded_at_epoch_seconds: 1_708_560_200,
        })
        .expect("first message should shred");

    let hold_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, PARTITION_MONTH_PRIMARY, vec![MESSAGE_ONE, MESSAGE_TWO]),
        )
        .expect("hold projection should succeed");
    assert_eq!(
        hold_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_LEGAL_HOLD_ACTIVE_REASON_CODE
    );
    let blocked_archive = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: NOW_MONTH_ID,
            active_retention_months: 1,
            object_storage_prefix: ARCHIVE_PREFIX.to_owned(),
        })
        .expect("archive due should succeed");
    assert!(blocked_archive.is_empty());

    m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_TWO.to_owned(),
            legal_hold_active: false,
        })
        .expect("legal hold release should apply");
    m8_registry
        .crypto_shred(DataLayerM8CryptoShredRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: MESSAGE_TWO.to_owned(),
            shredded_at_epoch_seconds: 1_708_560_220,
        })
        .expect("second message should shred after hold release");

    let final_projection = m10_registry
        .project_partition_shred_completeness_from_m8(
            &m8_registry,
            project_request(owner_did, PARTITION_MONTH_PRIMARY, vec![MESSAGE_ONE, MESSAGE_TWO]),
        )
        .expect("final projection should succeed");
    assert_eq!(
        final_projection.reason_code,
        DATA_LAYER_M10_COMPLIANCE_SHRED_COMPLETENESS_TRUE_REASON_CODE
    );
    assert!(final_projection.all_messages_shredded);

    let archived = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: NOW_MONTH_ID,
            active_retention_months: 1,
            object_storage_prefix: ARCHIVE_PREFIX.to_owned(),
        })
        .expect("archive due should succeed");
    assert_eq!(archived.len(), 1);
    assert_eq!(archived[0].partition_name, "messages_2024_01");
}
