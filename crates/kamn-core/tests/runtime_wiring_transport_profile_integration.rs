use kamn_core::{
    build_runtime_wiring, build_runtime_wiring_with_transport_profile, libp2p_feature_gate_name,
    resolve_libp2p_compile_mode, Libp2pCompileMode, NodeConfig, NodeRole, RuntimeTransportProfile,
    SyncMode,
};

fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: gossip_enabled,
        sync_mode: SyncMode::Fast,
    }
}

#[test]
fn integration_default_runtime_wiring_uses_in_memory_transport_markers() {
    let wiring = build_runtime_wiring(&config_for(NodeRole::Processor, true));
    let components = wiring.all_components();

    assert!(components.contains(&"p2p-transport-profile:in-memory-deterministic"));
    assert!(components.contains(&"p2p-in-memory-transport-fallback"));
    assert!(!components.contains(&"p2p-transport-profile:libp2p-live"));
    assert!(!components.contains(&"p2p-live-libp2p-provider"));
}

#[test]
fn integration_live_transport_profile_emits_provider_and_compile_mode_markers() {
    let wiring = build_runtime_wiring_with_transport_profile(
        &config_for(NodeRole::Processor, true),
        RuntimeTransportProfile::Libp2pLive,
    );
    let components = wiring.all_components();

    assert!(components.contains(&"p2p-transport-profile:libp2p-live"));
    assert!(components.contains(&"p2p-live-libp2p-provider"));
    assert!(components.contains(&resolve_libp2p_compile_mode().marker_component()));
    assert!(!components.contains(&"p2p-in-memory-transport-fallback"));
}

#[test]
fn integration_gossip_disabled_runtime_wiring_uses_disabled_marker_only() {
    let default_wiring = build_runtime_wiring(&config_for(NodeRole::Processor, false));
    let live_wiring = build_runtime_wiring_with_transport_profile(
        &config_for(NodeRole::Processor, false),
        RuntimeTransportProfile::Libp2pLive,
    );

    for components in [default_wiring.all_components(), live_wiring.all_components()] {
        assert!(components.contains(&"gossip-transport-disabled"));
        assert!(!components.contains(&"p2p-discovery"));
        assert!(!components.contains(&"p2p-gossip-transport"));
        assert!(!components.contains(&"p2p-libp2p-swarm-stack"));
        assert!(!components.contains(&"p2p-transport-profile:in-memory-deterministic"));
        assert!(!components.contains(&"p2p-transport-profile:libp2p-live"));
        assert!(!components.contains(&"p2p-live-libp2p-provider"));
        assert!(!components.contains(&"p2p-in-memory-transport-fallback"));
    }
}

#[test]
fn integration_role_specific_runtime_wiring_components_stay_stable() {
    let processor = build_runtime_wiring(&config_for(NodeRole::Processor, true));
    let listener = build_runtime_wiring(&config_for(NodeRole::Listener, true));
    let approver = build_runtime_wiring(&config_for(NodeRole::Approver, true));

    assert_eq!(
        processor.role_components,
        vec!["mempool", "executor", "block-producer", "consensus-validator"]
    );
    assert_eq!(
        listener.role_components,
        vec!["external-listener", "event-normalizer"]
    );
    assert_eq!(
        approver.role_components,
        vec!["quorum-approver", "outbound-authorizer"]
    );
}

#[test]
fn integration_feature_gate_name_and_compile_mode_marker_align() {
    assert_eq!(libp2p_feature_gate_name(), "libp2p-live-transport");

    let compile_mode = resolve_libp2p_compile_mode();
    if cfg!(feature = "libp2p-live-transport") {
        assert_eq!(compile_mode, Libp2pCompileMode::NativeLibp2p);
        assert_eq!(compile_mode.marker_component(), "p2p-live-libp2p-provider:native");
    } else {
        assert_eq!(compile_mode, Libp2pCompileMode::ContractOnly);
        assert_eq!(
            compile_mode.marker_component(),
            "p2p-live-libp2p-provider:contract-only"
        );
    }
}
