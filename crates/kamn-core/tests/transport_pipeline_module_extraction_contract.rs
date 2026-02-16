use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/src/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read src/{path}: {error}");
    })
}

#[test]
fn p2p_transport_module_boundaries_decompose_live_runtime_sections() {
    let p2p_transport_rs = read_repo_file("p2p_transport.rs");
    assert!(
        p2p_transport_rs.contains("mod p2p_transport_live;"),
        "p2p_transport.rs should declare extracted p2p_transport_live module"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct Libp2pLivePeerLifecycleTransport {"),
        "p2p_transport.rs should not keep inline Libp2pLivePeerLifecycleTransport definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub enum Libp2pLiveRuntimeBackend {"),
        "p2p_transport.rs should not keep inline Libp2pLiveRuntimeBackend definition"
    );

    let p2p_transport_live_rs = read_repo_file("p2p_transport/p2p_transport_live.rs");
    assert!(
        p2p_transport_live_rs.contains("pub struct Libp2pLivePeerLifecycleTransport {"),
        "p2p_transport_live module should own Libp2pLivePeerLifecycleTransport"
    );
    assert!(
        p2p_transport_live_rs.contains("pub enum Libp2pLiveRuntimeBackend {"),
        "p2p_transport_live module should own Libp2pLiveRuntimeBackend"
    );
}

#[test]
fn block_pipeline_module_boundaries_decompose_ingress_and_store_sections() {
    let block_pipeline_rs = read_repo_file("block_pipeline.rs");
    assert!(
        block_pipeline_rs.contains("mod block_pipeline_support;"),
        "block_pipeline.rs should declare extracted block_pipeline_support module"
    );
    assert!(
        !block_pipeline_rs.contains("pub struct GossipIngressAdapter;"),
        "block_pipeline.rs should not keep inline GossipIngressAdapter definition"
    );
    assert!(
        !block_pipeline_rs.contains("pub struct InMemoryCanonicalCommitStore {"),
        "block_pipeline.rs should not keep inline InMemoryCanonicalCommitStore definition"
    );

    let block_pipeline_support_rs = read_repo_file("block_pipeline/block_pipeline_support.rs");
    assert!(
        block_pipeline_support_rs.contains("pub struct GossipIngressAdapter;"),
        "block_pipeline_support module should own GossipIngressAdapter"
    );
    assert!(
        block_pipeline_support_rs.contains("pub struct InMemoryCanonicalCommitStore {"),
        "block_pipeline_support module should own InMemoryCanonicalCommitStore"
    );
}
