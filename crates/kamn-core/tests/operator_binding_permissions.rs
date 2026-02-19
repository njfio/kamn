use kamn_core::{
    OperatorBindingAction, OperatorBindingEngine, OperatorBindingError, OperatorBindingProof,
};
use std::collections::BTreeSet;

fn permission_set(values: &[OperatorBindingAction]) -> BTreeSet<OperatorBindingAction> {
    values.iter().copied().collect()
}

fn valid_proof(operator_did: &str) -> OperatorBindingProof {
    OperatorBindingProof {
        type_name: "Ed25519Signature2020".to_owned(),
        created: "2026-02-07T20:00:00Z".to_owned(),
        verification_method: format!("{operator_did}#keys-1"),
        proof_value: "z58proofvalue".to_owned(),
    }
}

#[test]
fn optional_proof_binding_allows_configure_permission() {
    let mut engine = OperatorBindingEngine::new();

    engine
        .register_binding(
            "kamn:did:agent:agent-1",
            "kamn:did:human:operator-1",
            None,
            permission_set(&[
                OperatorBindingAction::Configure,
                OperatorBindingAction::ReadHistory,
            ]),
        )
        .expect("binding should register");

    engine
        .authorize(
            "kamn:did:agent:agent-1",
            "kamn:did:human:operator-1",
            OperatorBindingAction::Configure,
        )
        .expect("configure should be authorized");
}

#[test]
fn missing_permission_is_rejected_with_typed_error() {
    let mut engine = OperatorBindingEngine::new();
    engine
        .register_binding(
            "kamn:did:agent:agent-2",
            "kamn:did:human:operator-2",
            Some(valid_proof("kamn:did:human:operator-2")),
            permission_set(&[OperatorBindingAction::ReadHistory]),
        )
        .expect("binding should register");

    assert_eq!(
        engine.authorize(
            "kamn:did:agent:agent-2",
            "kamn:did:human:operator-2",
            OperatorBindingAction::Revoke,
        ),
        Err(OperatorBindingError::UnauthorizedAction {
            operator_did: "kamn:did:human:operator-2".to_owned(),
            action: OperatorBindingAction::Revoke,
        })
    );
}

#[test]
fn integration_revoked_binding_blocks_read_history() {
    let mut engine = OperatorBindingEngine::new();
    engine
        .register_binding(
            "kamn:did:agent:agent-3",
            "kamn:did:human:operator-3",
            Some(OperatorBindingProof {
                type_name: "Ed25519Signature2020".to_owned(),
                created: "2026-02-07T20:00:00Z".to_owned(),
                verification_method: "kamn:did:human:operator-3#keys-9".to_owned(),
                proof_value: "z58proofvalue-3".to_owned(),
            }),
            permission_set(&[
                OperatorBindingAction::Revoke,
                OperatorBindingAction::ReadHistory,
            ]),
        )
        .expect("binding should register");

    engine
        .revoke_binding("kamn:did:agent:agent-3", "kamn:did:human:operator-3")
        .expect("revoke should succeed");

    assert_eq!(
        engine.authorize(
            "kamn:did:agent:agent-3",
            "kamn:did:human:operator-3",
            OperatorBindingAction::ReadHistory,
        ),
        Err(OperatorBindingError::RevokedBinding {
            agent_did: "kamn:did:agent:agent-3".to_owned(),
            operator_did: "kamn:did:human:operator-3".to_owned(),
        })
    );
}

#[test]
fn regression_unbound_operator_cannot_read_history() {
    let engine = OperatorBindingEngine::new();

    // Regression: #231
    assert_eq!(
        engine.authorize(
            "kamn:did:agent:agent-99",
            "kamn:did:human:operator-99",
            OperatorBindingAction::ReadHistory,
        ),
        Err(OperatorBindingError::MissingBinding {
            agent_did: "kamn:did:agent:agent-99".to_owned(),
            operator_did: "kamn:did:human:operator-99".to_owned(),
        })
    );
}

#[test]
fn invalid_operator_did_surfaces_reason_code_contract() {
    let mut engine = OperatorBindingEngine::new();

    assert_eq!(
        engine.register_binding(
            "kamn:did:agent:agent-11",
            "bad-did",
            Some(valid_proof("kamn:did:human:operator-11")),
            permission_set(&[OperatorBindingAction::Configure]),
        ),
        Err(OperatorBindingError::InvalidOperatorDid {
            field: "operator_did",
            reason_code: "operator_binding_invalid_operator_did",
            detail: "invalid human did prefix: bad-did".to_owned(),
        })
    );
}
