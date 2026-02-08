use kamn_core::{
    canonical_state_key, ClassificationPolicy, ClassificationStatus, DataClassificationEngine,
    DataClassificationError, DataClassificationLevel, WriteDomain, WriteRequestContext, WriteTag,
};
use std::collections::{BTreeMap, BTreeSet};

fn policy() -> ClassificationPolicy {
    let mut minimum_by_domain = BTreeMap::new();
    minimum_by_domain.insert(WriteDomain::Messages, DataClassificationLevel::Internal);
    minimum_by_domain.insert(WriteDomain::Tasks, DataClassificationLevel::Internal);
    minimum_by_domain.insert(WriteDomain::Escrows, DataClassificationLevel::Sensitive);
    minimum_by_domain.insert(WriteDomain::Reputation, DataClassificationLevel::Public);

    let mut required_tags_by_level = BTreeMap::new();
    required_tags_by_level.insert(
        DataClassificationLevel::Sensitive,
        ["contains-sensitive".to_owned()].into_iter().collect(),
    );
    required_tags_by_level.insert(
        DataClassificationLevel::Restricted,
        [
            "contains-sensitive".to_owned(),
            "contains-restricted".to_owned(),
        ]
        .into_iter()
        .collect(),
    );

    ClassificationPolicy {
        minimum_by_domain,
        required_tags_by_level,
    }
}

#[test]
fn sensitive_write_with_required_tag_is_authorized() {
    let mut engine = DataClassificationEngine::new(policy()).expect("engine should construct");
    let key = engine
        .authorize_write(&WriteRequestContext {
            domain: WriteDomain::Messages,
            record_id: "msg-42".to_owned(),
            actor: "kamn:did:agent:writer-1".to_owned(),
            tag: WriteTag {
                level: DataClassificationLevel::Sensitive,
                tags: ["contains-sensitive".to_owned()].into_iter().collect(),
            },
        })
        .expect("write should be authorized");

    assert_eq!(
        key,
        canonical_state_key("kamn.messages", "record", "msg-42")
            .expect("canonical key should compute")
    );
}

#[test]
fn missing_required_tags_is_rejected_with_typed_error() {
    let mut engine = DataClassificationEngine::new(policy()).expect("engine should construct");

    assert_eq!(
        engine.authorize_write(&WriteRequestContext {
            domain: WriteDomain::Messages,
            record_id: "msg-43".to_owned(),
            actor: "kamn:did:agent:writer-2".to_owned(),
            tag: WriteTag {
                level: DataClassificationLevel::Restricted,
                tags: ["contains-sensitive".to_owned()].into_iter().collect(),
            },
        }),
        Err(DataClassificationError::MissingRequiredTags {
            level: DataClassificationLevel::Restricted,
            missing: ["contains-restricted".to_owned()].into_iter().collect(),
        })
    );
}

#[test]
fn integration_domain_minimum_and_status_surface_are_enforced() {
    let mut engine = DataClassificationEngine::new(policy()).expect("engine should construct");

    assert_eq!(
        engine.authorize_write(&WriteRequestContext {
            domain: WriteDomain::Escrows,
            record_id: "escrow-1".to_owned(),
            actor: "kamn:did:agent:writer-3".to_owned(),
            tag: WriteTag {
                level: DataClassificationLevel::Internal,
                tags: BTreeSet::new(),
            },
        }),
        Err(DataClassificationError::ClassificationBelowDomainMinimum {
            domain: WriteDomain::Escrows,
            required: DataClassificationLevel::Sensitive,
            provided: DataClassificationLevel::Internal,
        })
    );

    assert_eq!(
        engine
            .status_for(
                WriteDomain::Escrows,
                "escrow-1",
                WriteTag {
                    level: DataClassificationLevel::Internal,
                    tags: BTreeSet::new(),
                },
            )
            .expect("status should resolve"),
        ClassificationStatus {
            domain: WriteDomain::Escrows,
            record_id: "escrow-1".to_owned(),
            minimum_level: DataClassificationLevel::Sensitive,
            provided_level: DataClassificationLevel::Internal,
            missing_tags: Vec::new(),
            authorized: false,
        }
    );
}

#[test]
fn regression_untagged_sensitive_write_is_blocked() {
    let mut engine = DataClassificationEngine::new(policy()).expect("engine should construct");

    // Regression: #157
    assert_eq!(
        engine.authorize_write(&WriteRequestContext {
            domain: WriteDomain::Messages,
            record_id: "msg-99".to_owned(),
            actor: "kamn:did:agent:writer-4".to_owned(),
            tag: WriteTag {
                level: DataClassificationLevel::Sensitive,
                tags: BTreeSet::new(),
            },
        }),
        Err(DataClassificationError::UntaggedSensitiveWrite(
            "msg-99".to_owned()
        ))
    );
}
