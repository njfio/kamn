use super::DOC;

pub fn assert_doc_contains_all(markers: &[&str]) {
    for marker in markers {
        assert!(DOC.contains(marker), "missing doc marker: {marker}");
    }
}

pub fn assert_doc_contains_prefixed_entries(prefix: &str, codes: &[&str]) {
    for code in codes {
        let marker = format!("{prefix}.{code}=");
        assert!(DOC.contains(&marker), "missing doc marker: {marker}");
    }
}
