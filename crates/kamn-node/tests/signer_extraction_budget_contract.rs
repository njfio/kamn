const SIGNER_MODULE_SOURCE: &str = include_str!("../src/signer.rs");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

const SIGNER_RS_MAX_LINES: usize = 950;

fn signer_module_line_count() -> usize {
    SIGNER_MODULE_SOURCE.lines().count()
}

#[test]
fn signer_module_budget_stays_within_threshold() {
    let actual = signer_module_line_count();
    assert!(
        actual <= SIGNER_RS_MAX_LINES,
        "signer.rs line budget exceeded: actual={actual}, max={SIGNER_RS_MAX_LINES}"
    );
}

#[test]
fn signer_module_declares_required_extraction_ownership_markers() {
    assert!(
        SIGNER_MODULE_SOURCE.contains("mod managed_backend;"),
        "signer.rs must declare managed_backend module"
    );
    assert!(
        SIGNER_MODULE_SOURCE.contains("mod nonce;"),
        "signer.rs must declare nonce module"
    );
    assert!(
        SIGNER_MODULE_SOURCE.contains("mod signer_policy;"),
        "signer.rs must declare signer_policy module"
    );
    assert!(
        SIGNER_MODULE_SOURCE.contains("pub(crate) use managed_backend::{"),
        "signer.rs must re-export managed backend public API from extracted module"
    );
    assert!(
        SIGNER_MODULE_SOURCE.contains("pub(crate) use nonce::resolve_kolme_live_nonce;"),
        "signer.rs must re-export nonce resolver from extracted module"
    );
    assert!(
        SIGNER_MODULE_SOURCE.contains("pub(crate) use signer_policy::{"),
        "signer.rs must re-export signer policy APIs from extracted module"
    );
    assert!(
        !SIGNER_MODULE_SOURCE.contains("fn sign_kolme_live_managed_external_message("),
        "signer.rs must not re-inline managed backend signing implementation"
    );
    assert!(
        !SIGNER_MODULE_SOURCE.contains("fn resolve_kolme_live_nonce("),
        "signer.rs must not re-inline nonce resolver implementation"
    );
}

#[test]
fn docs_ci_strategy_declares_signer_extraction_budget_guard_policy() {
    assert!(
        CI_STRATEGY_DOC.contains("### Signer Extraction Budget Guard"),
        "ci strategy docs must declare signer extraction budget guard section"
    );
    assert!(
        CI_STRATEGY_DOC.contains("signer_extraction_budget_guard_status=active"),
        "ci strategy docs must declare signer extraction budget guard status marker"
    );
    assert!(
        CI_STRATEGY_DOC.contains("signer_rs_max_lines=950"),
        "ci strategy docs must declare signer.rs max line budget marker"
    );
    assert!(
        CI_STRATEGY_DOC.contains(
            "cargo test -p kamn-node --test signer_extraction_budget_contract -- --nocapture"
        ),
        "ci strategy docs must include signer extraction budget guard command"
    );
}
