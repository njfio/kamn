use super::{DOC, OPS_DOC};

pub(crate) fn assert_reason_code_present_in_docs_and_ops(reason_code: &str, label: &str) {
    assert!(
        !reason_code.trim().is_empty(),
        "reason code entries must stay non-empty"
    );
    assert!(
        DOC.contains(reason_code),
        "ci strategy docs missing {label} reason code marker: {reason_code}"
    );
    assert!(
        OPS_DOC.contains(reason_code),
        "ops docs missing {label} reason code marker: {reason_code}"
    );
}

pub(crate) fn assert_remediation_marker_in_docs_and_ops(
    prefix: &str,
    reason_code: &str,
    label: &str,
) {
    let marker = format!("{prefix}.{reason_code}=");
    assert!(
        DOC.contains(&marker),
        "missing {label} remediation marker for {reason_code}"
    );
    assert!(
        OPS_DOC.contains(&marker),
        "ops docs missing {label} remediation marker for {reason_code}"
    );
}
