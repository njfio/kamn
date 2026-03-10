pub(crate) const DOC: &str = include_str!("../../../../docs/foundation/node-runtime-cli.md");
pub(crate) const KOLME_RUNTIME_COMMIT_DOC: &str =
    include_str!("../../../../docs/architecture/kolme-runtime-commit.md");
pub(crate) const RUNTIME_PROCESSOR_HA_DOC: &str =
    include_str!("../../../../docs/foundation/runtime-processor-ha.md");
pub(crate) const R42_API_SHELL_SURFACE_AUDIT_DOC: &str =
    include_str!("../../../../docs/review/r42-api-shell-surface-audit.md");
pub(crate) const SIGNER_LIFECYCLE_DOC: &str =
    include_str!("../../../../docs/architecture/signer-lifecycle.md");

pub(crate) fn assert_markers(document: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(
            document.contains(marker),
            "missing marker in {label}: {marker}"
        );
    }
}
