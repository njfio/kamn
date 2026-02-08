const DOC: &str = include_str!("../../../docs/foundation/trust-score-engine.md");

#[test]
fn doc_contains_prd_formula_components() {
    assert!(DOC.contains("## PRD 8.2 Formula Mapping"));
    assert!(DOC.contains("base_score = 500"));
    assert!(DOC.contains("delivery_component"));
    assert!(DOC.contains("response_component"));
    assert!(DOC.contains("dispute_penalty"));
    assert!(DOC.contains("volume_bonus"));
    assert!(DOC.contains("endorsement_bonus"));
}

#[test]
fn doc_contains_bounds_and_versioning_controls() {
    assert!(DOC.contains("## Deterministic Bounds and Versioning"));
    assert!(DOC.contains("TRUST_SCORE_ENGINE_VERSION"));
    assert!(DOC.contains("clamped to `0..=1000`"));
    assert!(DOC.contains("delivery_rate and dispute_rate must be within `0.0..=1.0`"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test trust_score_engine"));
    assert!(DOC.contains("cargo clippy -p kamn-core -- -D warnings"));
}

#[test]
fn regression_requires_1000ms_bucket_boundary_rule() {
    // Regression: #213
    assert!(DOC.contains("`1000ms` remains in the highest response bucket."));
}
