const ENGINEERING_HARDENING_WAVE_DOC: &str =
    include_str!("../../../docs/planning/engineering-hardening-wave.md");
const ARCHITECTURE_MODULE_MAP_DOC: &str =
    include_str!("../../../docs/architecture/kamn-core-module-map.md");
const RUSTDOC_PUBLISHING_DOC: &str = include_str!("../../../docs/developer/rustdoc-publishing.md");
const README: &str = include_str!("../../../README.md");

#[test]
fn engineering_hardening_wave_doc_declares_missing_docs_policy_contract() {
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("check_kamn_core_missing_docs_policy.sh"));
    assert!(
        ENGINEERING_HARDENING_WAVE_DOC.contains("run_kamn_core_rustdoc_artifact_contract_lane.sh")
    );
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("check_kamn_core_rustdoc_artifact_policy.sh"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC
        .contains("cargo test -p kamn-core --test lifecycle_evidence_property_matrix"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC
        .contains("cargo test -p kamn-core --test concurrency_task_terminal_race"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("kamn-core"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("#![warn(missing_docs)]"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("docs/architecture/kamn-core-module-map.md"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC
        .contains("docs/architecture/kamn-core-module-map.md#contributor-entrypoint-matrix"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("docs/developer/rustdoc-publishing.md"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("Regression: #1526"));
    assert!(ENGINEERING_HARDENING_WAVE_DOC.contains("Regression: #1527"));
}

#[test]
fn architecture_module_map_documents_runtime_flow_and_entrypoints() {
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("KAMN Core Module Map"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("## Ownership Matrix"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("## Runtime Flow (Condensed)"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("## Contributor Entrypoint Matrix"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("### Governance and Operator Control Plane"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("### Storage, Content, and Compliance"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("### Runtime, State, and Safety"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("governance_workflow"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("operator_dashboard_api"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("redaction_compliance"));
    assert!(ARCHITECTURE_MODULE_MAP_DOC.contains("crates/kamn-core/src/lib.rs"));
}

#[test]
fn rustdoc_publishing_doc_declares_bounded_command_surface() {
    assert!(RUSTDOC_PUBLISHING_DOC.contains("cargo doc -p kamn-core --no-deps"));
    assert!(RUSTDOC_PUBLISHING_DOC
        .contains("RUSTDOCFLAGS=\"-D warnings\" cargo doc -p kamn-core --no-deps"));
    assert!(RUSTDOC_PUBLISHING_DOC.contains("target/doc"));
    assert!(RUSTDOC_PUBLISHING_DOC.contains("run_kamn_core_rustdoc_artifact_contract_lane.sh"));
    assert!(RUSTDOC_PUBLISHING_DOC.contains("check_kamn_core_rustdoc_artifact_policy.sh"));
    assert!(RUSTDOC_PUBLISHING_DOC.contains("kamn.ci.kamn-core-rustdoc-artifact-report.v1"));
    assert!(RUSTDOC_PUBLISHING_DOC
        .contains("docs/architecture/kamn-core-module-map.md#contributor-entrypoint-matrix"));
}

#[test]
fn readme_references_engineering_hardening_wave_and_policy_checker() {
    assert!(README.contains("docs/planning/engineering-hardening-wave.md"));
    assert!(README.contains("docs/planning/engineering-hardening-wave.md#commands"));
    assert!(README.contains("check_kamn_core_missing_docs_policy.sh"));
    assert!(README.contains("docs/architecture/kamn-core-module-map.md"));
    assert!(README.contains("docs/architecture/kamn-core-module-map.md#ownership-matrix"));
    assert!(
        README.contains("docs/architecture/kamn-core-module-map.md#contributor-entrypoint-matrix")
    );
    assert!(README.contains("docs/developer/rustdoc-publishing.md"));
    assert!(README.contains("docs/developer/rustdoc-publishing.md#contract-enforcement"));
}
