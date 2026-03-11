use kamn_governance::{
    OperatorActionOutcome, OperatorActionServiceError, OperatorBindingAction, OperatorBindingEngine,
    OperatorBindingError, OperatorBindingProof, PermissionedOperatorActionService,
};

fn proof_for(operator_did: &str) -> OperatorBindingProof {
    OperatorBindingProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: "2026-02-07T20:00:00Z".to_owned(),
        verification_method: format!("{operator_did}#keys-1"),
        proof_value: "z58proof".to_owned(),
    }
}

fn service_with_permissions(permissions: &[OperatorBindingAction]) -> PermissionedOperatorActionService {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-3",
            "kamn:did:human:op-3",
            Some(proof_for("kamn:did:human:op-3")),
            permissions.iter().copied().collect(),
        )
        .expect("binding should register");
    PermissionedOperatorActionService::new(bindings)
}

#[test]
fn read_history_denied_records_audit_entry_fail_closed() {
    let mut service = service_with_permissions(&[OperatorBindingAction::Configure]);
    assert_eq!(
        service.read_history("kamn:did:agent:ops-3", "kamn:did:human:op-3", 9),
        Err(OperatorActionServiceError::Binding(
            OperatorBindingError::UnauthorizedAction {
                operator_did: "kamn:did:human:op-3".to_owned(),
                action: OperatorBindingAction::ReadHistory,
            },
        ))
    );
    let audit_log = service.audit_log();
    assert_eq!(audit_log.len(), 1);
    assert_eq!(audit_log[0].target, "audit_log");
    assert_eq!(audit_log[0].outcome, OperatorActionOutcome::Denied);
}

#[test]
fn revoke_binding_denied_records_audit_entry_fail_closed() {
    let mut service = service_with_permissions(&[OperatorBindingAction::Configure]);
    assert_eq!(
        service.revoke_binding("kamn:did:agent:ops-3", "kamn:did:human:op-3", 10),
        Err(OperatorActionServiceError::Binding(
            OperatorBindingError::UnauthorizedAction {
                operator_did: "kamn:did:human:op-3".to_owned(),
                action: OperatorBindingAction::Revoke,
            },
        ))
    );
    let audit_log = service.audit_log();
    assert_eq!(audit_log.len(), 1);
    assert_eq!(audit_log[0].target, "binding");
    assert_eq!(audit_log[0].outcome, OperatorActionOutcome::Denied);
}
