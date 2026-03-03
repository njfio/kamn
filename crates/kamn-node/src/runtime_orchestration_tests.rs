use super::*;

#[test]
fn unit_build_runtime_execution_id_formats_mode_chain_and_role() {
    let execution_id = build_runtime_execution_id(RuntimeMode::full(), "kamn-mainnet", "sequencer");
    assert_eq!(execution_id, "node-runtime:full:kamn-mainnet:sequencer");
}
