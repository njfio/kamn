const FOUNDATION_DOC: &str =
    include_str!("../../../docs/foundation/kolme-runtime-commit-client.md");
const DEVNET_DOC: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");
const CI_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const README_DOC: &str = include_str!("../../../README.md");

#[test]
fn docs_include_onchain_lifecycle_evidence_bundle_contract_markers() {
    let docs = [FOUNDATION_DOC, DEVNET_DOC, CI_DOC, README_DOC];
    for doc in docs {
        assert!(doc.contains("run_onchain_lifecycle_evidence_bundle_lane.sh"));
        assert!(doc.contains("check_onchain_lifecycle_evidence_policy.py"));
        assert!(doc.contains("run_onchain_lifecycle_evidence_contract_lane.sh"));
        assert!(doc.contains("aggregate_bundle_lineage_mismatch"));
        assert!(doc.contains("finality_lineage_missing"));
        assert!(doc.contains("recovery_lineage_missing"));
        assert!(doc.contains("Regression: #3249"));
    }
}
