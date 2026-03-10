use super::super::support::*;

pub struct CiExclusionContext {
    pub fast_workflow: String,
    pub ci_tools: String,
    pub ci_tools_fast_mode: String,
    pub strategy_doc: String,
}

pub fn load_ci_exclusion_context() -> CiExclusionContext {
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    CiExclusionContext {
        fast_workflow: read_text(FAST_WORKFLOW),
        ci_tools_fast_mode: extract_fast_mode_block(&ci_tools),
        strategy_doc: read_text(CI_STRATEGY_DOC),
        ci_tools,
    }
}

pub fn assert_excluded(haystack: &str, marker: &str, message: &str) {
    assert!(!haystack.contains(marker), "{message}");
}

pub fn assert_fast_gate_exclusion(
    context: &CiExclusionContext,
    workflow_marker: &str,
    fast_mode_marker: &str,
    lane_label: &str,
) {
    assert_excluded(
        &context.fast_workflow,
        workflow_marker,
        &format!("{lane_label} must remain excluded from ci-fast-gate"),
    );
    assert_excluded(
        &context.ci_tools_fast_mode,
        fast_mode_marker,
        &format!("{lane_label} must remain excluded from ci-tools fast mode"),
    );
}

pub fn assert_ci_tools_surface_and_doc(
    context: &CiExclusionContext,
    ci_tools_markers: &[&str],
    ci_tools_label: &str,
    doc_marker: &str,
    lane_label: &str,
) {
    assert_contains_all(&context.ci_tools, ci_tools_markers, ci_tools_label);
    assert!(
        context.strategy_doc.contains(doc_marker),
        "ci strategy doc missing exclusion marker for {lane_label}"
    );
}
