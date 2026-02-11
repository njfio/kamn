use kamn_kolme::{
    resolve_finality, EchoTransport, FinalityState, PassthroughCodec, RuntimeCommitPipeline,
};

#[test]
fn integration_scaffold_pipeline_surface_is_callable() {
    let pipeline = RuntimeCommitPipeline::new(PassthroughCodec, EchoTransport);
    let output = pipeline
        .submit("http://localhost/runtime-commit", b"hello-kolme", 1, false)
        .expect("pipeline should succeed");
    assert_eq!(output, b"hello-kolme");
}

#[test]
fn regression_issue_1719_scaffold_finality_contract_is_stable() {
    // Regression: #1719
    let resolution = resolve_finality(0, 1, false);
    assert_eq!(resolution.state(), FinalityState::Pending);
    assert!(!resolution.is_final());
}
