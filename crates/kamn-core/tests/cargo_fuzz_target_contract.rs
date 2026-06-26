use kamn_core::{
    parse_signature_profile_metadata, KolmeApiBroadcastResponse, KolmeApiNextNonceResponse,
    BASELINE_SIGNATURE_ALGORITHM, BASELINE_SIGNATURE_PROFILE_ID,
};
use std::path::Path;

const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const SEED_CORPUS_METADATA: &str =
    include_str!("../../../fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json");
const SIGNATURE_PROFILE_SEED_VALID: &str =
    include_str!("../../../fuzz/corpus/signature_profile_parser/seed-0001-baseline-signature.txt");
const SIGNATURE_PROFILE_SEED_MALFORMED: &str =
    include_str!("../../../fuzz/corpus/signature_profile_parser/seed-0002-malformed-signature.txt");
const KOLME_CODEC_SEED_VALID: &str = include_str!(
    "../../../fuzz/corpus/kolme_api_codec_parser/seed-0001-valid-next-nonce-response.json"
);
const KOLME_CODEC_SEED_MALFORMED: &str = include_str!(
    "../../../fuzz/corpus/kolme_api_codec_parser/seed-0002-malformed-broadcast-response.json"
);
const KOLME_FLAT_JSON_SEED_VALID: &str =
    include_str!("../../../fuzz/corpus/kolme_flat_json_parser/seed-0001-valid-flat-json.txt");
const KOLME_FLAT_JSON_SEED_MALFORMED: &str =
    include_str!("../../../fuzz/corpus/kolme_flat_json_parser/seed-0002-malformed-flat-json.txt");
const MESSAGE_ENVELOPE_TARGET: &str =
    include_str!("../../../fuzz/fuzz_targets/message_envelope_parser.rs");

#[test]
fn cargo_fuzz_package_and_targets_exist() {
    assert!(Path::new("../../fuzz/Cargo.toml").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/message_envelope_parser.rs").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/did_parser.rs").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/signature_profile_parser.rs").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/kolme_api_codec_parser.rs").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/kolme_flat_json_parser.rs").is_file());
}

#[test]
fn cargo_fuzz_seed_corpus_and_replay_metadata_exist() {
    assert!(Path::new("../../fuzz/corpus/message_envelope_parser/seed-0001-wire.txt").is_file());
    assert!(Path::new("../../fuzz/corpus/did_parser/seed-0001-valid-did.txt").is_file());
    assert!(Path::new(
        "../../fuzz/corpus/signature_profile_parser/seed-0001-baseline-signature.txt"
    )
    .is_file());
    assert!(Path::new(
        "../../fuzz/corpus/kolme_api_codec_parser/seed-0001-valid-next-nonce-response.json"
    )
    .is_file());
    assert!(
        Path::new("../../fuzz/corpus/kolme_flat_json_parser/seed-0001-valid-flat-json.txt")
            .is_file()
    );
    assert!(
        Path::new("../../fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json").is_file()
    );
}

#[test]
fn ci_strategy_contains_cargo_fuzz_boundary_markers() {
    assert!(CI_STRATEGY_DOC.contains("## Cargo Fuzz Parser Contract"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run message_envelope_parser"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run did_parser"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run signature_profile_parser"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run kolme_api_codec_parser"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run kolme_flat_json_parser"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_ci_smoke_max_seconds=120"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_local_heavy_max_seconds=900"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_local_heavy_excluded_from_ci_fast_gate=true"));
}

#[test]
fn regression_cargo_fuzz_seed_corpus_metadata_tracks_required_targets_and_seed_files() {
    // Regression: #4139
    assert!(SEED_CORPUS_METADATA.contains("\"schema\": \"kamn.runtime.cargo-fuzz-seed-corpus.v1\""));
    assert!(SEED_CORPUS_METADATA.contains("\"name\": \"message_envelope_parser\""));
    assert!(SEED_CORPUS_METADATA.contains("\"name\": \"did_parser\""));
    assert!(SEED_CORPUS_METADATA.contains("\"name\": \"signature_profile_parser\""));
    assert!(SEED_CORPUS_METADATA.contains("\"name\": \"kolme_api_codec_parser\""));
    assert!(SEED_CORPUS_METADATA.contains("\"name\": \"kolme_flat_json_parser\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-wire.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-invalid-recipient.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-valid-did.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-invalid-did.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-baseline-signature.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-malformed-signature.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-valid-next-nonce-response.json\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-malformed-broadcast-response.json\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-valid-flat-json.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-malformed-flat-json.txt\""));
}

#[test]
fn regression_cargo_fuzz_seed_corpus_metadata_tracks_parser_failure_taxonomy_markers() {
    // Regression: #4139
    assert!(SEED_CORPUS_METADATA.contains("\"failure_taxonomy_markers\""));
    assert!(SEED_CORPUS_METADATA.contains("\"invalid envelope type\""));
    assert!(SEED_CORPUS_METADATA.contains("\"invalid sender did\""));
    assert!(SEED_CORPUS_METADATA.contains("\"invalid recipient did\""));
    assert!(SEED_CORPUS_METADATA.contains("\"invalid agent did prefix\""));
    assert!(SEED_CORPUS_METADATA.contains("\"invalid kamn did prefix\""));
    assert!(SEED_CORPUS_METADATA.contains("\"invalid characters\""));
    assert!(SEED_CORPUS_METADATA.contains("\"missing sig prefix\""));
    assert!(SEED_CORPUS_METADATA.contains("\"empty algorithm segment\""));
    assert!(SEED_CORPUS_METADATA.contains("\"empty payload segments\""));
    assert!(SEED_CORPUS_METADATA.contains("\"missing required field: next_nonce\""));
    assert!(SEED_CORPUS_METADATA.contains("\"missing required field: txhash\""));
    assert!(SEED_CORPUS_METADATA.contains("\"field must be positive\""));
}

#[test]
fn regression_cargo_fuzz_seed_corpus_metadata_tracks_deterministic_seed_provenance_markers() {
    // Regression: #4140
    assert!(SEED_CORPUS_METADATA
        .contains("\"seed_provenance_version\": \"kamn.runtime.cargo-fuzz-seed-provenance.v1\""));
    assert!(SEED_CORPUS_METADATA.contains(
        "\"deterministic_seed_replay_key\": \"cargo_fuzz_seed_replay:message_envelope_parser:v1\""
    ));
    assert!(SEED_CORPUS_METADATA
        .contains("\"deterministic_seed_replay_key\": \"cargo_fuzz_seed_replay:did_parser:v1\""));
    assert!(SEED_CORPUS_METADATA.contains(
        "\"deterministic_seed_replay_key\": \"cargo_fuzz_seed_replay:signature_profile_parser:v1\""
    ));
    assert!(SEED_CORPUS_METADATA.contains(
        "\"deterministic_seed_replay_key\": \"cargo_fuzz_seed_replay:kolme_api_codec_parser:v1\""
    ));
    assert!(SEED_CORPUS_METADATA.contains(
        "\"deterministic_seed_replay_key\": \"cargo_fuzz_seed_replay:kolme_flat_json_parser:v1\""
    ));
    assert!(SEED_CORPUS_METADATA.contains("\"seed_budget_ci_smoke_max_seconds\": 120"));
    assert!(SEED_CORPUS_METADATA.contains("\"seed_budget_local_heavy_max_seconds\": 900"));
}

#[test]
fn regression_ci_strategy_contains_cargo_fuzz_seed_provenance_budget_markers() {
    // Regression: #4140
    assert!(CI_STRATEGY_DOC
        .contains("cargo_fuzz_seed_provenance_version=kamn.runtime.cargo-fuzz-seed-provenance.v1"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_seed_replay:message_envelope_parser:v1"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_seed_replay:did_parser:v1"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_seed_replay:signature_profile_parser:v1"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_seed_replay:kolme_api_codec_parser:v1"));
    assert!(CI_STRATEGY_DOC.contains("cargo_fuzz_seed_replay:kolme_flat_json_parser:v1"));
    assert!(
        CI_STRATEGY_DOC
            .contains("cargo_fuzz_seed_budget_markers_csv=seed_budget_ci_smoke_max_seconds,seed_budget_local_heavy_max_seconds")
    );
}

#[test]
fn regression_message_envelope_fuzz_target_declares_input_bounds_markers() {
    assert!(
        MESSAGE_ENVELOPE_TARGET.contains("fn bounded_utf8(data: &[u8], max_len: usize) -> String")
    );
    assert!(MESSAGE_ENVELOPE_TARGET.contains("let raw = bounded_utf8(data, 4096);"));
    assert!(CI_STRATEGY_DOC.contains("message_envelope_parser_input_bound_bytes=4096"));
    assert!(
        CI_STRATEGY_DOC.contains("message_envelope_parser_bound_scope=pre-envelope-construction")
    );
}

#[test]
fn regression_signature_profile_seed_corpus_replays_expected_parser_outcomes() {
    // Regression: #5938
    let valid_signature = SIGNATURE_PROFILE_SEED_VALID.trim();
    let malformed_signature = SIGNATURE_PROFILE_SEED_MALFORMED.trim();

    let metadata =
        parse_signature_profile_metadata(valid_signature).expect("valid signature seed must parse");
    assert_eq!(metadata.algorithm, BASELINE_SIGNATURE_ALGORITHM);
    assert_eq!(metadata.profile_id, BASELINE_SIGNATURE_PROFILE_ID);
    assert_eq!(parse_signature_profile_metadata(malformed_signature), None);
}

#[test]
fn regression_kolme_api_codec_seed_corpus_replays_expected_parser_outcomes() {
    // Regression: #5938
    let valid_response = KOLME_CODEC_SEED_VALID.trim();
    let malformed_response = KOLME_CODEC_SEED_MALFORMED.trim();

    let nonce = KolmeApiNextNonceResponse::parse_json(valid_response)
        .expect("valid nonce response seed should parse");
    assert_eq!(nonce.next_nonce, 21);
    assert_eq!(nonce.account_id.as_deref(), Some("account-fuzz-21"));

    assert!(KolmeApiNextNonceResponse::parse_json(malformed_response).is_err());
    assert!(KolmeApiBroadcastResponse::parse_json(malformed_response).is_err());
}

#[test]
fn regression_kolme_flat_json_seed_corpus_contains_expected_shape_markers() {
    // Regression: #6217
    let valid_seed = KOLME_FLAT_JSON_SEED_VALID.trim();
    let malformed_seed = KOLME_FLAT_JSON_SEED_MALFORMED.trim();
    assert!(valid_seed.starts_with('{'));
    assert!(valid_seed.contains("\"provider\""));
    assert!(malformed_seed.contains("7.2"));
    assert!(!malformed_seed.ends_with('}'));
}
