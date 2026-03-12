use crate::bridge_adapter::{BridgeAdapterError, BridgePlatform, PassThroughBridgeAdapter};

#[test]
fn constructor_rejects_invalid_bridge_agent_did() {
    assert_eq!(
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "not-a-did"),
        Err(BridgeAdapterError::InvalidDid {
            field: "bridge_agent_did",
            reason_code: "bridge_adapter_invalid_bridge_agent_did",
            detail: "invalid agent did prefix: not-a-did".to_owned(),
        })
    );
}
