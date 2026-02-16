const DOC: &str = include_str!("../../../docs/security/secure-coding.md");

#[test]
fn doc_contains_panic_path_reachability_and_unsafe_fallback_markers() {
    assert!(DOC.contains("# Secure Coding"));
    assert!(DOC.contains("panic_path_reachability_policy=fail_closed"));
    assert!(DOC.contains("unsafe_fallback_default_policy=fail_closed"));
    assert!(DOC.contains(
        "scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --output-json /tmp/no-production-expect-report.json"
    ));
    assert!(DOC.contains(
        "production_panic_path_violation_markers=.expect(,panic!,unreachable!,unsafe_env_fallback_default"
    ));
    assert!(
        DOC.contains("production_panic_path_violation_class=panic_reachability|unsafe_fallback")
    );
}
