use super::{OperatorActionOutcome, OperatorActionServiceError, PermissionedOperatorActionService};
use crate::operator_binding::{
    OperatorBindingAction, OperatorBindingEngine, OperatorBindingProof,
};
use std::collections::BTreeSet;

fn proof_for(operator_did: &str) -> OperatorBindingProof {
    OperatorBindingProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: "2026-02-07T20:00:00Z".to_owned(),
        verification_method: format!("{operator_did}#keys-1"),
        proof_value: "z58proof".to_owned(),
    }
}

fn permissions(values: &[OperatorBindingAction]) -> BTreeSet<OperatorBindingAction> {
    values.iter().copied().collect()
}

#[test]
fn configure_rejects_empty_key() {
    let mut service = PermissionedOperatorActionService::new(OperatorBindingEngine::new());
    assert_eq!(
        service.configure("kamn:did:agent:ops", "kamn:did:human:op", "", "on", 1),
        Err(OperatorActionServiceError::EmptyField("config_key"))
    );
}

#[test]
fn configure_denied_records_audit_entry() {
    let mut service = PermissionedOperatorActionService::new(OperatorBindingEngine::new());

    assert!(matches!(
        service.configure(
            "kamn:did:agent:ops-1",
            "kamn:did:human:op-1",
            "delivery.mode",
            "strict",
            7,
        ),
        Err(OperatorActionServiceError::Binding(_))
    ));

    let audit_log = service.audit_log();
    assert_eq!(audit_log.len(), 1);
    assert_eq!(audit_log[0].outcome, OperatorActionOutcome::Denied);
    assert_eq!(audit_log[0].target, "delivery.mode");
}

#[test]
fn read_history_allows_bound_operator_and_records_allowed_outcome() {
    let mut bindings = OperatorBindingEngine::new();
    bindings
        .register_binding(
            "kamn:did:agent:ops-2",
            "kamn:did:human:op-2",
            Some(proof_for("kamn:did:human:op-2")),
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
            "kamn:did:human:op-2",
            "delivery.mode",
            "strict",
            8,
        )
        .expect("configure should succeed");

    let history = service
        .read_history("kamn:did:agent:ops-2", "kamn:did:human:op-2", 9)
        .expect("history read should succeed");

    assert_eq!(history.len(), 2);
    assert_eq!(history[0].outcome, OperatorActionOutcome::Allowed);
    assert_eq!(history[1].outcome, OperatorActionOutcome::Allowed);
    assert_eq!(history[1].target, "audit_log");
}
