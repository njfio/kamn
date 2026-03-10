use super::super::DOC;

pub fn assert_doc_contains_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert!(DOC.contains(marker), "missing {label} marker: {marker}");
    }
}
