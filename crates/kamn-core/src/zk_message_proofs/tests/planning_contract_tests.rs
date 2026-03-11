use super::super::{
    evaluate_zk_option, phase4_baseline_options, ZkArchitectureOption, ZkDesignError,
    ZkEvaluationPolicy, ZkProofSystem, ZkVerificationTopology,
};

#[test]
fn transparent_policy_penalizes_trusted_setup_options() {
    let options = phase4_baseline_options();
    let result = evaluate_zk_option(&options[0], ZkEvaluationPolicy::default())
        .expect("evaluation should succeed");
    assert!(!result.feasible);
    assert!(result
        .risks
        .iter()
        .any(|risk| risk.code == "trusted-setup-policy"));
}

#[test]
fn option_validation_rejects_zero_proof_size() {
    let option = ZkArchitectureOption {
        name: "invalid".to_owned(),
        proof_system: ZkProofSystem::Plonkish,
        verification_topology: ZkVerificationTopology::ValidatorQuorum,
        trusted_setup_required: false,
        deterministic_witness_inputs: true,
        prover_latency_ms: 10,
        verifier_latency_ms: 10,
        proof_size_bytes: 0,
        supports_batching: true,
        estimated_engineering_weeks: 2,
    };
    let result = evaluate_zk_option(&option, ZkEvaluationPolicy::default());
    assert_eq!(
        result,
        Err(ZkDesignError::InvalidOption {
            option: "invalid".to_owned(),
            reason: "proof_size_bytes must be greater than zero".to_owned()
        })
    );
}
