use std::path::Path;

const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const SEED_CORPUS_METADATA: &str =
    include_str!("../../../fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json");

#[test]
fn cargo_fuzz_package_and_targets_exist() {
    assert!(Path::new("../../fuzz/Cargo.toml").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/message_envelope_parser.rs").is_file());
    assert!(Path::new("../../fuzz/fuzz_targets/did_parser.rs").is_file());
}

#[test]
fn cargo_fuzz_seed_corpus_and_replay_metadata_exist() {
    assert!(Path::new("../../fuzz/corpus/message_envelope_parser/seed-0001-wire.txt").is_file());
    assert!(Path::new("../../fuzz/corpus/did_parser/seed-0001-valid-did.txt").is_file());
    assert!(
        Path::new("../../fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json").is_file()
    );
}

#[test]
fn ci_strategy_contains_cargo_fuzz_boundary_markers() {
    assert!(CI_STRATEGY_DOC.contains("## Cargo Fuzz Parser Contract"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run message_envelope_parser"));
    assert!(CI_STRATEGY_DOC.contains("cargo fuzz run did_parser"));
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
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-wire.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-invalid-recipient.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0001-valid-did.txt\""));
    assert!(SEED_CORPUS_METADATA.contains("\"seed-0002-invalid-did.txt\""));
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
    assert!(SEED_CORPUS_METADATA.contains("\"seed_budget_ci_smoke_max_seconds\": 120"));
    assert!(SEED_CORPUS_METADATA.contains("\"seed_budget_local_heavy_max_seconds\": 900"));
}

#[test]
fn regression_ci_strategy_contains_cargo_fuzz_seed_provenance_budget_markers() {
    // Regression: #4140
    assert!(CI_STRATEGY_DOC
        .contains("cargo_fuzz_seed_provenance_version=kamn.runtime.cargo-fuzz-seed-provenance.v1"));
    assert!(CI_STRATEGY_DOC.contains(
        "cargo_fuzz_seed_replay_keys_csv=cargo_fuzz_seed_replay:message_envelope_parser:v1,cargo_fuzz_seed_replay:did_parser:v1"
    ));
    assert!(
        CI_STRATEGY_DOC
            .contains("cargo_fuzz_seed_budget_markers_csv=seed_budget_ci_smoke_max_seconds,seed_budget_local_heavy_max_seconds")
    );
}
