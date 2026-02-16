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
    assert!(DOC.contains(
        "panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "panic_replacement_reason_codes_csv=scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default"
    ));
    assert!(DOC.contains("panic_replacement_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "panic_replacement_reason_class=stable|panic_reachability|unsafe_fallback|mixed|configuration"
    ));
    assert!(DOC.contains("runtime_panic_replacement_evidence_status=verified|violation"));
    assert!(DOC.contains("runtime_panic_replacement_evidence_violation_count=<n>"));
    assert!(DOC.contains("runtime_panic_replacement_evidence_files_csv=none|<csv>"));
    assert!(DOC.contains(
        "runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv"
    ));
}
