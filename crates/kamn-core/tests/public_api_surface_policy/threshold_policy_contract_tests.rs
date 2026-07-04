use crate::support::models::PolicyStatus;
use crate::support::{compute_report_with_policy, module_source_paths, repo_path};
use std::path::Path;

#[test]
fn public_api_surface_policy_enforces_warn_fail_contract() {
    let (report, thresholds, status, _reason_codes, _rendered) = compute_report_with_policy();
    let public_items_delta = report.public_items_delta;
    let fail_total_delta_max = thresholds.fail_total_delta_max;
    let status_marker = status.as_marker();
    assert!(matches!(
        status,
        PolicyStatus::Within | PolicyStatus::Warn | PolicyStatus::ExceptionApplied
    ));
    assert!(
        public_items_delta <= fail_total_delta_max
            || matches!(status, PolicyStatus::ExceptionApplied),
        "delta={public_items_delta} fail_max={fail_total_delta_max} status={status_marker}"
    );
}

#[test]
fn module_source_paths_exclude_cfg_test_sources() {
    let paths = module_source_paths("bootstrap", &repo_path("src"));

    assert!(
        paths.iter().any(|path| path.ends_with("bootstrap.rs")),
        "module root source should remain counted"
    );
    assert!(
        !paths.iter().any(|path| has_test_only_component(path)),
        "cfg(test) source paths must not count as public API: {paths:?}"
    );
}

fn has_test_only_component(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "tests")
}
