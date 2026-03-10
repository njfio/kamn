use super::super::DOC;

pub(super) fn assert_doc_contains(marker: &str, label: &str) {
    assert!(DOC.contains(marker), "missing {label} marker: {marker}");
}

pub(super) fn assert_doc_contains_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert_doc_contains(marker, label);
    }
}
