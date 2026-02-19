const DOC: &str = include_str!("../../../docs/testing/invariant-and-fuzz-strategy.md");

#[test]
fn doc_contains_live_transport_replay_tamper_contract_commands() {
    assert!(DOC.contains("run_live_transport_replay_tamper_contract_lane.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_fast_lane.sh"));
    assert!(DOC.contains("run_live_transport_replay_tamper_deep_lane.sh"));
    assert!(DOC.contains("check_live_transport_replay_tamper_policy.sh"));
    assert!(DOC.contains("kamn.sdk.live-transport-replay-tamper-evidence.v1"));
}

#[test]
fn regression_requires_live_transport_replay_tamper_contract_markers() {
    // Regression: #1380
    assert!(DOC.contains("/tmp/live-transport-replay-tamper-contract-report.json"));
    assert!(DOC.contains("bundle-file /tmp/live-transport-replay-tamper-contract-report.json"));
}

#[test]
fn regression_requires_lifecycle_property_replay_metadata_markers() {
    // Regression: #1605
    assert!(DOC.contains("kamn.runtime.lifecycle-property-replay-metadata.v1"));
    assert!(DOC.contains("generated_sequence_bounds"));
    assert!(DOC.contains("executed_cases"));
}

#[test]
fn regression_requires_input_mutation_targeted_smoke_markers() {
    // Regression: #1607
    assert!(DOC.contains("--target envelope"));
    assert!(DOC.contains("--target did"));
    assert!(DOC.contains("kamn.runtime.input-mutation-replay-metadata.v1"));
    assert!(DOC.contains("input_mutation_envelope_seed:v1"));
    assert!(DOC.contains("input_mutation_did_seed:v1"));
}

#[test]
fn regression_requires_coverage_guided_input_mutation_markers() {
    // Regression: #2693
    assert!(DOC.contains("run_input_mutation_coverage_guided_contract_lane.sh"));
    assert!(DOC.contains("run_input_mutation_coverage_guided_contract_lane.sh --target envelope"));
    assert!(DOC.contains("run_input_mutation_coverage_guided_contract_lane.sh --target did"));
    assert!(DOC.contains("run_input_mutation_coverage_guided_deep_lane.sh"));
    assert!(DOC.contains("kamn.runtime.input-mutation-coverage-guided-contract-report.v1"));
    assert!(DOC.contains("kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1"));
    assert!(DOC.contains("input_mutation_coverage_guided_replay:v1"));
    assert!(DOC.contains("minimal_failing_seed_prefix"));
    assert!(DOC.contains("KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS"));
    assert!(DOC.contains("KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS"));
    assert!(DOC.contains("KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY"));
    assert!(DOC.contains("excluded from `ci-fast-gate`"));
}

#[test]
fn regression_requires_parser_failure_taxonomy_markers() {
    // Regression: #4139
    assert!(DOC.contains(
        "input_mutation_coverage_guided_parser_failure_taxonomy_version=kamn.runtime.input-mutation-coverage-guided-parser-failure-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "input_mutation_coverage_guided_parser_failure_codes_csv=invalid_envelope_type,invalid_sender_did,invalid_recipient_did,invalid_message_type,invalid_encryption_algorithm,empty_body,invalid_proof_purpose,proof_verification_method_mismatch,invalid_agent_did_prefix,invalid_kamn_did_prefix,invalid_characters,missing_method_specific_id"
    ));
}
