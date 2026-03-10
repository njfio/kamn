pub(super) fn assert_doc_contains_all(markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            super::super::DOC.contains(marker),
            "missing {label} marker: {marker}"
        );
    }
}
