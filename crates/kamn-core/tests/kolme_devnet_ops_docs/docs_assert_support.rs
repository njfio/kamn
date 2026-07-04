use super::{DEPLOY_COMPAT, PLAN};

pub fn assert_plan_contains_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert!(PLAN.contains(marker), "missing {label} marker: {marker}");
    }
}

pub fn assert_deploy_contains_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            DEPLOY_COMPAT.contains(marker),
            "missing {label} marker: {marker}"
        );
    }
}
