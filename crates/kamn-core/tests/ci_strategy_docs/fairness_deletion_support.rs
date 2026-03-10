use super::{DOC, OPS_DOC};

pub(crate) fn assert_reason_codes_non_empty(reason_codes: Vec<&str>, label: &str) {
    for reason_code in reason_codes {
        assert!(
            !reason_code.trim().is_empty(),
            "{label} reason code entries must stay non-empty"
        );
    }
}

pub(crate) fn assert_doc_remediation_markers(prefix: &str, reason_codes: Vec<&str>, label: &str) {
    for reason_code in reason_codes {
        assert!(
            DOC.contains(&format!("{prefix}.{reason_code}=")),
            "missing {label} remediation marker for {reason_code}"
        );
    }
}

pub(crate) fn assert_docs_and_ops_remediation_markers(
    prefix: &str,
    reason_codes: Vec<&str>,
    label: &str,
) {
    for reason_code in reason_codes {
        let marker = format!("{prefix}.{reason_code}=");
        assert!(DOC.contains(&marker), "missing {label} remediation marker for {reason_code}");
        assert!(OPS_DOC.contains(&marker), "ops docs missing {label} remediation marker for {reason_code}");
    }
}
