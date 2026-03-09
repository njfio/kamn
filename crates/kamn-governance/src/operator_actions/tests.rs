use super::{OperatorActionServiceError, PermissionedOperatorActionService};
use crate::operator_binding::OperatorBindingEngine;

#[test]
fn configure_rejects_empty_key() {
    let mut service = PermissionedOperatorActionService::new(OperatorBindingEngine::new());
    assert_eq!(
        service.configure("kamn:did:agent:ops", "kamn:did:human:op", "", "on", 1),
        Err(OperatorActionServiceError::EmptyField("config_key"))
    );
}
