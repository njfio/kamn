use crate::operator_binding::{
    OperatorBindingAction, OperatorBindingEngine, OperatorBindingError, OperatorBindingProof,
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

fn register_binding(
    engine: &mut OperatorBindingEngine,
    agent_did: &str,
    operator_did: &str,
    permissions: &[OperatorBindingAction],
) {
    engine
        .register_binding(
            agent_did,
            operator_did,
            Some(proof_for(operator_did)),
            crate_permissions(permissions),
        )
        .expect("binding should register");
}

fn revoked_engine() -> OperatorBindingEngine {
    let mut engine = OperatorBindingEngine::new();
    register_binding(
        &mut engine,
        "kamn:did:agent:agent-11",
        "kamn:did:human:operator-11",
        &[
            OperatorBindingAction::Configure,
            OperatorBindingAction::Revoke,
        ],
    );
    engine
        .revoke_binding("kamn:did:agent:agent-11", "kamn:did:human:operator-11")
        .expect("revoke should succeed");
    engine
}

fn crate_permissions(values: &[OperatorBindingAction]) -> BTreeSet<OperatorBindingAction> {
    permissions(values)
}

#[test]
fn register_rejects_invalid_operator_did() {
    let mut engine = OperatorBindingEngine::new();
    assert_eq!(
        engine.register_binding(
            "kamn:did:agent:agent-1",
            "did:example:operator",
            None,
            permissions(&[OperatorBindingAction::Configure]),
        ),
        Err(OperatorBindingError::InvalidOperatorDid {
            field: "operator_did",
            reason_code: "operator_binding_invalid_operator_did",
            detail: "invalid human did prefix: did:example:operator".to_owned(),
        })
    );
}

#[test]
fn register_rejects_invalid_proof_verification_method() {
    let mut engine = OperatorBindingEngine::new();
    assert_eq!(
        engine.register_binding(
            "kamn:did:agent:agent-2",
            "kamn:did:human:operator-2",
            Some(OperatorBindingProof {
                type_name: "Ed25519Signature2020".to_owned(),
                created: "2026-02-07T20:00:00Z".to_owned(),
                verification_method: "kamn:did:human:other#keys-1".to_owned(),
                proof_value: "z58proof".to_owned(),
            }),
            permissions(&[OperatorBindingAction::Configure]),
        ),
        Err(OperatorBindingError::ProofVerificationMethodMismatch {
            expected_prefix: "kamn:did:human:operator-2#".to_owned(),
            actual: "kamn:did:human:other#keys-1".to_owned(),
        })
    );
}

#[test]
fn revoke_requires_revoke_permission() {
    let mut engine = OperatorBindingEngine::new();
    register_binding(
        &mut engine,
        "kamn:did:agent:agent-3",
        "kamn:did:human:operator-3",
        &[OperatorBindingAction::ReadHistory],
    );

    assert_eq!(
        engine.revoke_binding("kamn:did:agent:agent-3", "kamn:did:human:operator-3"),
        Err(OperatorBindingError::UnauthorizedAction {
            operator_did: "kamn:did:human:operator-3".to_owned(),
            action: OperatorBindingAction::Revoke,
        })
    );
}

#[test]
fn authorize_allows_granted_action() {
    let mut engine = OperatorBindingEngine::new();
    register_binding(
        &mut engine,
        "kamn:did:agent:agent-4",
        "kamn:did:human:operator-4",
        &[
            OperatorBindingAction::Configure,
            OperatorBindingAction::ReadHistory,
            OperatorBindingAction::Revoke,
        ],
    );

    engine
        .authorize(
            "kamn:did:agent:agent-4",
            "kamn:did:human:operator-4",
            OperatorBindingAction::ReadHistory,
        )
        .expect("authorization should pass");
}

#[test]
fn authorize_rejects_missing_binding() {
    let engine = OperatorBindingEngine::new();
    assert_eq!(
        engine.authorize(
            "kamn:did:agent:agent-9",
            "kamn:did:human:operator-9",
            OperatorBindingAction::ReadHistory,
        ),
        Err(OperatorBindingError::MissingBinding {
            agent_did: "kamn:did:agent:agent-9".to_owned(),
            operator_did: "kamn:did:human:operator-9".to_owned(),
        })
    );
}

#[test]
fn register_rejects_duplicate_binding() {
    let mut engine = OperatorBindingEngine::new();
    register_binding(
        &mut engine,
        "kamn:did:agent:agent-10",
        "kamn:did:human:operator-10",
        &[OperatorBindingAction::Configure],
    );

    assert_eq!(
        engine.register_binding(
            "kamn:did:agent:agent-10",
            "kamn:did:human:operator-10",
            Some(proof_for("kamn:did:human:operator-10")),
            permissions(&[OperatorBindingAction::Configure]),
        ),
        Err(OperatorBindingError::DuplicateBinding {
            agent_did: "kamn:did:agent:agent-10".to_owned(),
            operator_did: "kamn:did:human:operator-10".to_owned(),
        })
    );
}

#[test]
fn authorize_rejects_revoked_binding_fail_closed() {
    let engine = revoked_engine();
    assert_eq!(
        engine.authorize(
            "kamn:did:agent:agent-11",
            "kamn:did:human:operator-11",
            OperatorBindingAction::Configure,
        ),
        Err(OperatorBindingError::RevokedBinding {
            agent_did: "kamn:did:agent:agent-11".to_owned(),
            operator_did: "kamn:did:human:operator-11".to_owned(),
        })
    );
}
