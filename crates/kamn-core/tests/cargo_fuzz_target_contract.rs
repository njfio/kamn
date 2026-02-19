use std::path::Path;

const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

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
