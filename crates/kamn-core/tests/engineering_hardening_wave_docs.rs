const ENGINEERING_HARDENING_WAVE_DOC: &str =
    include_str!("../../../docs/planning/engineering-hardening-wave.md");
const README: &str = include_str!("../../../README.md");

#[test]
fn engineering_hardening_wave_doc_declares_missing_docs_policy_contract() {
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("check_kamn_core_missing_docs_policy.sh"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("kamn-core"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("#![warn(missing_docs)]"));
}

#[test]
fn readme_references_engineering_hardening_wave_and_policy_checker() {
    assert!(README.contains("docs/planning/engineering-hardening-wave.md"));
    assert!(README.contains("check_kamn_core_missing_docs_policy.sh"));
}
