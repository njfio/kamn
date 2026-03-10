use super::CHECKLIST;

pub fn assert_checklist_contains_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert!(CHECKLIST.contains(marker), "missing {label} marker: {marker}");
    }
}

pub fn assert_checklist_omits_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert!(!CHECKLIST.contains(marker), "unexpected {label} marker: {marker}");
    }
}
