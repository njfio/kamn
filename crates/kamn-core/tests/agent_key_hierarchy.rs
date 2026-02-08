use kamn_core::{AgentKeyHierarchy, AgentKeyHierarchyError, KeyRole};

#[test]
fn hierarchy_tracks_identity_signing_and_agreement_keys() {
    let hierarchy = AgentKeyHierarchy::new("id:key:v1", "sig:key:v1", "agr:key:v1")
        .expect("hierarchy should initialize");

    assert_eq!(
        hierarchy
            .current_key(KeyRole::Identity)
            .expect("identity key"),
        "id:key:v1"
    );
    assert_eq!(
        hierarchy
            .current_key(KeyRole::Signing)
            .expect("signing key"),
        "sig:key:v1"
    );
    assert_eq!(
        hierarchy
            .current_key(KeyRole::Agreement)
            .expect("agreement key"),
        "agr:key:v1"
    );
}

#[test]
fn signing_and_agreement_rotation_update_role_bindings() {
    let mut hierarchy = AgentKeyHierarchy::new("id:key:v1", "sig:key:v1", "agr:key:v1")
        .expect("hierarchy should initialize");

    hierarchy
        .rotate_signing_key("sig:key:v2")
        .expect("signing rotation should succeed");
    hierarchy
        .rotate_agreement_key("agr:key:v2")
        .expect("agreement rotation should succeed");

    assert_eq!(
        hierarchy
            .current_key(KeyRole::Signing)
            .expect("signing key"),
        "sig:key:v2"
    );
    assert_eq!(
        hierarchy
            .current_key(KeyRole::Agreement)
            .expect("agreement key"),
        "agr:key:v2"
    );
}

#[test]
fn ephemeral_session_keys_are_registered_and_queryable() {
    let mut hierarchy = AgentKeyHierarchy::new("id:key:v1", "sig:key:v1", "agr:key:v1")
        .expect("hierarchy should initialize");

    hierarchy
        .register_ephemeral("session-a", "eph:key:1", 1_000)
        .expect("ephemeral registration should succeed");
    let session_key = hierarchy
        .ephemeral_key("session-a")
        .expect("session key should exist");
    assert_eq!(session_key.key_id, "eph:key:1");
    assert_eq!(session_key.expires_at_secs, 1_000);
}

#[test]
fn duplicate_ephemeral_session_is_rejected() {
    let mut hierarchy = AgentKeyHierarchy::new("id:key:v1", "sig:key:v1", "agr:key:v1")
        .expect("hierarchy should initialize");
    hierarchy
        .register_ephemeral("session-a", "eph:key:1", 1_000)
        .expect("ephemeral registration should succeed");

    assert_eq!(
        hierarchy.register_ephemeral("session-a", "eph:key:2", 2_000),
        Err(AgentKeyHierarchyError::DuplicateSession(
            "session-a".to_owned()
        ))
    );
}

#[test]
fn retired_ephemeral_keys_are_not_retrievable() {
    let mut hierarchy = AgentKeyHierarchy::new("id:key:v1", "sig:key:v1", "agr:key:v1")
        .expect("hierarchy should initialize");
    hierarchy
        .register_ephemeral("session-z", "eph:key:z", 1_000)
        .expect("ephemeral registration should succeed");
    hierarchy
        .retire_ephemeral("session-z")
        .expect("retire should succeed");

    // Regression: #123
    assert_eq!(
        hierarchy.ephemeral_key("session-z"),
        Err(AgentKeyHierarchyError::SessionNotFound(
            "session-z".to_owned()
        ))
    );
}
