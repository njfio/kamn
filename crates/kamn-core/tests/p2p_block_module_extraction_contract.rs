use std::fs;

fn read_src_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/src/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read src/{path}: {error}");
    })
}

#[test]
fn p2p_transport_module_extraction_contract_declares_swarm_stack_module() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        p2p_transport_rs.contains("mod swarm_stack;"),
        "p2p_transport.rs should declare extracted swarm_stack module"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_declares_adapter_module() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        p2p_transport_rs.contains("mod adapter;"),
        "p2p_transport.rs should declare extracted adapter module"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_moves_swarm_stack_types_out_of_root_file() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        !p2p_transport_rs.contains("pub struct P2pSwarmDeterministicConfig {"),
        "p2p_transport.rs should not keep inline P2pSwarmDeterministicConfig definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct P2pSwarmHarnessTask {"),
        "p2p_transport.rs should not keep inline P2pSwarmHarnessTask definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct LiveTransportReconnectPolicy {"),
        "p2p_transport.rs should not keep inline LiveTransportReconnectPolicy definition"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_moves_adapter_types_out_of_root_file() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        !p2p_transport_rs.contains("pub struct PeerDiscoveryRecord {"),
        "p2p_transport.rs should not keep inline PeerDiscoveryRecord definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct PeerGossipFrame {"),
        "p2p_transport.rs should not keep inline PeerGossipFrame definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub trait PeerLifecycleTransport {"),
        "p2p_transport.rs should not keep inline PeerLifecycleTransport definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct InMemoryPeerLifecycleTransport {"),
        "p2p_transport.rs should not keep inline InMemoryPeerLifecycleTransport definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct UdpPeerLifecycleTransport {"),
        "p2p_transport.rs should not keep inline UdpPeerLifecycleTransport definition"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_keeps_swarm_stack_impls_in_new_module() {
    let swarm_stack_rs = read_src_file("p2p_transport/swarm_stack.rs");
    assert!(
        swarm_stack_rs.contains("pub struct P2pSwarmDeterministicConfig {"),
        "swarm_stack module should own P2pSwarmDeterministicConfig"
    );
    assert!(
        swarm_stack_rs.contains("pub struct P2pSwarmHarnessTask {"),
        "swarm_stack module should own P2pSwarmHarnessTask"
    );
    assert!(
        swarm_stack_rs.contains("pub struct LiveTransportReconnectPolicy {"),
        "swarm_stack module should own LiveTransportReconnectPolicy"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_keeps_adapter_impls_in_new_module() {
    let adapter_rs = read_src_file("p2p_transport/adapter.rs");
    assert!(
        adapter_rs.contains("pub struct PeerDiscoveryRecord {"),
        "adapter module should own PeerDiscoveryRecord"
    );
    assert!(
        adapter_rs.contains("pub struct PeerGossipFrame {"),
        "adapter module should own PeerGossipFrame"
    );
    assert!(
        adapter_rs.contains("pub trait PeerLifecycleTransport {"),
        "adapter module should own PeerLifecycleTransport"
    );
    assert!(
        adapter_rs.contains("pub struct InMemoryPeerLifecycleTransport {"),
        "adapter module should own InMemoryPeerLifecycleTransport"
    );
    assert!(
        adapter_rs.contains("pub struct UdpPeerLifecycleTransport {"),
        "adapter module should own UdpPeerLifecycleTransport"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_declares_native_runtime_module() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        p2p_transport_rs.contains("mod native_runtime;"),
        "p2p_transport.rs should declare extracted native_runtime module"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_declares_runtime_event_module() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        p2p_transport_rs.contains("mod runtime_event;"),
        "p2p_transport.rs should declare extracted runtime_event module"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_moves_native_runtime_loop_out_of_root_file() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        !p2p_transport_rs.contains("struct Libp2pNativeRuntimeAdapterLoop {"),
        "p2p_transport.rs should not keep inline Libp2pNativeRuntimeAdapterLoop definition"
    );
    assert!(
        !p2p_transport_rs.contains("enum Libp2pNativeRuntimeAdapterLoopCommand {"),
        "p2p_transport.rs should not keep inline Libp2pNativeRuntimeAdapterLoopCommand definition"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_moves_runtime_event_types_out_of_root_file() {
    let p2p_transport_rs = read_src_file("p2p_transport.rs");
    assert!(
        !p2p_transport_rs.contains("pub enum Libp2pRuntimeEventKind {"),
        "p2p_transport.rs should not keep inline Libp2pRuntimeEventKind definition"
    );
    assert!(
        !p2p_transport_rs.contains("pub struct Libp2pRuntimeEvent {"),
        "p2p_transport.rs should not keep inline Libp2pRuntimeEvent definition"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_keeps_native_runtime_impls_in_new_module() {
    let native_runtime_rs = read_src_file("p2p_transport/native_runtime.rs");
    assert!(
        native_runtime_rs.contains("pub(super) struct Libp2pNativeRuntimeAdapterLoop {"),
        "native_runtime module should own Libp2pNativeRuntimeAdapterLoop"
    );
    assert!(
        native_runtime_rs.contains("enum Libp2pNativeRuntimeAdapterLoopCommand {"),
        "native_runtime module should own Libp2pNativeRuntimeAdapterLoopCommand"
    );
}

#[test]
fn p2p_transport_module_extraction_contract_keeps_runtime_event_impls_in_new_module() {
    let runtime_event_rs = read_src_file("p2p_transport/runtime_event.rs");
    assert!(
        runtime_event_rs.contains("pub enum Libp2pRuntimeEventKind {"),
        "runtime_event module should own Libp2pRuntimeEventKind"
    );
    assert!(
        runtime_event_rs.contains("pub struct Libp2pRuntimeEvent {"),
        "runtime_event module should own Libp2pRuntimeEvent"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_declares_fork_choice_module() {
    let block_pipeline_rs = read_src_file("block_pipeline.rs");
    assert!(
        block_pipeline_rs.contains("mod fork_choice;"),
        "block_pipeline.rs should declare extracted fork_choice module"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_moves_fork_choice_types_out_of_root_file() {
    let block_pipeline_rs = read_src_file("block_pipeline.rs");
    assert!(
        !block_pipeline_rs.contains("pub trait ForkChoiceHook {"),
        "block_pipeline.rs should not keep inline ForkChoiceHook definition"
    );
    assert!(
        !block_pipeline_rs.contains("pub struct DeterministicCompetingBranchForkChoiceHook {"),
        "block_pipeline.rs should not keep inline DeterministicCompetingBranchForkChoiceHook definition"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_keeps_fork_choice_impls_in_new_module() {
    let fork_choice_rs = read_src_file("block_pipeline/fork_choice.rs");
    assert!(
        fork_choice_rs.contains("pub trait ForkChoiceHook {"),
        "fork_choice module should own ForkChoiceHook"
    );
    assert!(
        fork_choice_rs.contains("pub struct DeterministicCompetingBranchForkChoiceHook {"),
        "fork_choice module should own DeterministicCompetingBranchForkChoiceHook"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_declares_commit_store_module() {
    let block_pipeline_rs = read_src_file("block_pipeline.rs");
    assert!(
        block_pipeline_rs.contains("mod commit_store;"),
        "block_pipeline.rs should declare extracted commit_store module"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_declares_evidence_module() {
    let block_pipeline_rs = read_src_file("block_pipeline.rs");
    assert!(
        block_pipeline_rs.contains("mod evidence;"),
        "block_pipeline.rs should declare extracted evidence module"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_moves_commit_store_types_out_of_root_file() {
    let block_pipeline_rs = read_src_file("block_pipeline.rs");
    assert!(
        !block_pipeline_rs.contains("pub trait CanonicalCommitStore {"),
        "block_pipeline.rs should not keep inline CanonicalCommitStore definition"
    );
    assert!(
        !block_pipeline_rs.contains("pub struct SqliteCanonicalCommitStore {"),
        "block_pipeline.rs should not keep inline SqliteCanonicalCommitStore definition"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_moves_evidence_types_out_of_root_file() {
    let block_pipeline_rs = read_src_file("block_pipeline.rs");
    assert!(
        !block_pipeline_rs.contains("pub struct CanonicalReplayEvidenceBundle {"),
        "block_pipeline.rs should not keep inline CanonicalReplayEvidenceBundle definition"
    );
    assert!(
        !block_pipeline_rs.contains("pub struct TransportConvergenceEvidenceBundle {"),
        "block_pipeline.rs should not keep inline TransportConvergenceEvidenceBundle definition"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_keeps_commit_store_impls_in_new_module() {
    let commit_store_rs = read_src_file("block_pipeline/commit_store.rs");
    assert!(
        commit_store_rs.contains("pub trait CanonicalCommitStore {"),
        "commit_store module should own CanonicalCommitStore"
    );
    assert!(
        commit_store_rs.contains("pub struct SqliteCanonicalCommitStore {"),
        "commit_store module should own SqliteCanonicalCommitStore"
    );
}

#[test]
fn block_pipeline_module_extraction_contract_keeps_evidence_impls_in_new_module() {
    let evidence_rs = read_src_file("block_pipeline/evidence.rs");
    assert!(
        evidence_rs.contains("pub struct CanonicalReplayEvidenceBundle {"),
        "evidence module should own CanonicalReplayEvidenceBundle"
    );
    assert!(
        evidence_rs.contains("pub struct TransportConvergenceEvidenceBundle {"),
        "evidence module should own TransportConvergenceEvidenceBundle"
    );
}
