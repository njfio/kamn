use kamn_core::{
    build_runtime_wiring_with_transport_profile, libp2p_feature_gate_name,
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
fn unit_resolve_libp2p_compile_mode_matches_feature_flag_state() {
    let mode = resolve_libp2p_compile_mode();
    if cfg!(feature = "libp2p-live-transport") {
        assert_eq!(mode, Libp2pCompileMode::NativeLibp2p);
    } else {
        assert_eq!(mode, Libp2pCompileMode::ContractOnly);
    }
}

#[test]
fn functional_libp2p_feature_gate_name_stays_stable() {
    assert_eq!(libp2p_feature_gate_name(), "libp2p-live-transport");
}

#[test]
fn integration_live_transport_wiring_includes_compile_mode_marker() {
    let wiring = build_runtime_wiring_with_transport_profile(
        &config_for(NodeRole::Processor, true),
        RuntimeTransportProfile::Libp2pLive,
    );
    assert!(wiring
        .all_components()
        .contains(&resolve_libp2p_compile_mode().marker_component()));
}

#[test]
fn regression_compile_mode_marker_matches_feature_flag_state() {
    // Regression: #3651
    let marker = resolve_libp2p_compile_mode().marker_component();
    if cfg!(feature = "libp2p-live-transport") {
        assert_eq!(marker, "p2p-live-libp2p-provider:native");
    } else {
        assert_eq!(marker, "p2p-live-libp2p-provider:contract-only");
    }
}
