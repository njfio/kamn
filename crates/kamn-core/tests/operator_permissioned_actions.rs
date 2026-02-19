use kamn_core::{
    OperatorActionServiceError, OperatorBindingAction, OperatorBindingEngine, OperatorBindingError,
    OperatorBindingProof, PermissionedOperatorActionService,
};
use std::collections::BTreeSet;

fn permissions(actions: &[OperatorBindingAction]) -> BTreeSet<OperatorBindingAction> {
    actions.iter().copied().collect()
}

fn proof_for(operator_did: &str) -> OperatorBindingProof {
    OperatorBindingProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: "2026-02-08T12:00:00Z".to_owned(),
        verification_method: format!("{operator_did}#keys-1"),
        proof_value: "z58proof".to_owned(),
    }
}

#[test]
fn operator_actions_reject_empty_configuration_keys() {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-1",
            "kamn:did:human:alice-1",
            Some(proof_for("kamn:did:human:alice-1")),
            permissions(&[OperatorBindingAction::Configure]),
        )
        .expect("binding should register");

    let mut service = PermissionedOperatorActionService::new(bindings);
    assert_eq!(
        service.configure(
            "kamn:did:agent:ops-1",
            "kamn:did:human:alice-1",
            "",
            "on",
            1_716_100_000,
        ),
        Err(OperatorActionServiceError::EmptyField("config_key"))
    );
}

#[test]
fn operator_actions_authorized_configure_updates_state_and_audit() {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-2",
            "kamn:did:human:alice-2",
            Some(proof_for("kamn:did:human:alice-2")),
            permissions(&[
                OperatorBindingAction::Configure,
                OperatorBindingAction::ReadHistory,
            ]),
        )
        .expect("binding should register");

    let mut service = PermissionedOperatorActionService::new(bindings);
    service
        .configure(
            "kamn:did:agent:ops-2",
            "kamn:did:human:alice-2",
            "maintenance_mode",
            "enabled",
            1_716_100_100,
        )
        .expect("configure should pass");
    assert_eq!(
        service.setting("kamn:did:agent:ops-2", "maintenance_mode"),
        Some("enabled".to_owned())
    );
    assert_eq!(service.audit_log().len(), 1);
}

#[test]
fn operator_actions_revoke_binding_blocks_follow_up_configure() {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-3",
            "kamn:did:human:alice-3",
            Some(proof_for("kamn:did:human:alice-3")),
            permissions(&[
                OperatorBindingAction::Configure,
                OperatorBindingAction::Revoke,
            ]),
        )
        .expect("binding should register");

    let mut service = PermissionedOperatorActionService::new(bindings);
    service
        .revoke_binding(
            "kamn:did:agent:ops-3",
            "kamn:did:human:alice-3",
            1_716_100_200,
        )
        .expect("revoke should pass");
    assert!(matches!(
        service.configure(
            "kamn:did:agent:ops-3",
            "kamn:did:human:alice-3",
            "maintenance_mode",
            "enabled",
            1_716_100_201,
        ),
        Err(OperatorActionServiceError::Binding(_))
    ));
}

#[test]
fn operator_actions_integration_read_history_requires_binding_permission() {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-4",
            "kamn:did:human:alice-4",
            Some(proof_for("kamn:did:human:alice-4")),
            permissions(&[
                OperatorBindingAction::Configure,
                OperatorBindingAction::ReadHistory,
            ]),
        )
        .expect("binding should register");

    let mut service = PermissionedOperatorActionService::new(bindings);
    service
        .configure(
            "kamn:did:agent:ops-4",
            "kamn:did:human:alice-4",
            "throttle_profile",
            "strict",
            1_716_100_300,
        )
        .expect("configure should pass");

    let history = service
        .read_history(
            "kamn:did:agent:ops-4",
            "kamn:did:human:alice-4",
            1_716_100_301,
        )
        .expect("history read should pass");
    assert!(!history.is_empty());
}

#[test]
fn operator_actions_regression_unauthorized_operator_cannot_mutate_settings() {
    // Regression: #199
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-5",
            "kamn:did:human:alice-5",
            Some(proof_for("kamn:did:human:alice-5")),
            permissions(&[OperatorBindingAction::ReadHistory]),
        )
        .expect("binding should register");

    let mut service = PermissionedOperatorActionService::new(bindings);
    assert!(matches!(
        service.configure(
            "kamn:did:agent:ops-5",
            "kamn:did:human:alice-5",
            "maintenance_mode",
            "enabled",
            1_716_100_400,
        ),
        Err(OperatorActionServiceError::Binding(_))
    ));
    assert_eq!(
        service.setting("kamn:did:agent:ops-5", "maintenance_mode"),
        None
    );
}

#[test]
fn invalid_agent_did_surfaces_reason_code_contract() {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-6",
            "kamn:did:human:alice-6",
            Some(proof_for("kamn:did:human:alice-6")),
            permissions(&[OperatorBindingAction::Configure]),
        )
        .expect("binding should register");
    let mut service = PermissionedOperatorActionService::new(bindings);

    assert_eq!(
        service.configure(
            "bad-did",
            "kamn:did:human:alice-6",
            "maintenance_mode",
            "enabled",
            1_716_100_500,
        ),
        Err(OperatorActionServiceError::Binding(
            OperatorBindingError::InvalidAgentDid {
                field: "agent_did",
                reason_code: "operator_binding_invalid_agent_did",
                detail: "invalid agent did prefix: bad-did".to_owned(),
            }
        ))
    );
}
