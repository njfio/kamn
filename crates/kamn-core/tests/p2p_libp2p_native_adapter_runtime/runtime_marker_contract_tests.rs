use super::support::*;

#[test]
fn functional_libp2p_native_backend_selection_marker_is_stable() {
    assert_eq!(resolve_libp2p_live_runtime_backend(), Libp2pLiveRuntimeBackend::NativeSocket);
    assert_eq!(resolve_libp2p_live_runtime_backend().marker(), "native-libp2p-swarm");
}

#[test]
fn functional_libp2p_native_adapter_loop_marker_is_stable() {
    let transport = new_transport(
        "peer-native-loop-marker",
        "/ip4/127.0.0.1/tcp/9560",
        unique_bootstrap_seed().as_str(),
    );
    assert_eq!(transport.native_runtime_loop_marker(), "libp2p-runtime-adapter-loop");
}
