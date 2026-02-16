use std::collections::BTreeMap;

const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const PRODUCTION_NEXT_STEPS_DOC: &str =
    include_str!("../../../docs/plans/2026-02-14-production-service-next-steps.md");

const FALLBACK_SECRET_VIOLATION_REASON: &str = "fallback_signer_secret_present_violation";

const REQUIRED_POLICY_MARKER_KEYS: &[&str] = &[
    "signer_secret_redaction_regression_guard_status",
    "signer_secret_redaction_policy",
    "signer_secret_lifecycle_policy_contract_status",
    "signer_secret_lifecycle_policy_contract_version",
    "signer_secret_lifecycle_forbidden_reason_code",
    "signer_secret_lifecycle_required_markers_csv",
];

const REQUIRED_LIFECYCLE_CSV_ENTRIES: &[&str] = &[
    "signer_secret_redaction_regression_guard_status",
    "signer_secret_redaction_policy",
    FALLBACK_SECRET_VIOLATION_REASON,
    "signer_secret_source_precedence_violation",
];

const REQUIRED_CI_DOC_MARKERS: &[&str] = &[
    "signer_secret_redaction_regression_guard_status=active",
    "signer_secret_redaction_policy=raw_private_key_value_never_emitted",
    "signer_secret_lifecycle_policy_contract_status=active",
    "signer_secret_lifecycle_policy_contract_version=v1",
    "signer_secret_lifecycle_forbidden_reason_code=fallback_signer_secret_present_violation",
    "signer_secret_lifecycle_required_markers_csv=signer_secret_redaction_regression_guard_status,signer_secret_redaction_policy,fallback_signer_secret_present_violation,signer_secret_source_precedence_violation",
];

const REQUIRED_PLAN_DOC_MARKERS: &[&str] = &[
    "signer_secret_hardening_closure_chain=#3911->#3915->(#3916,#3917)",
    "signer_secret_lifecycle_policy_contract_status=active",
    "signer_secret_lifecycle_policy_contract_version=v1",
    "signer_secret_lifecycle_docs_contract_status=active",
    "signer_secret_lifecycle_contract_guard_command=cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract -- --nocapture",
];

#[test]
fn policy_checker_rejects_fallback_secret_violation_reason_code() {
    let markers = complete_policy_markers();
    let error = evaluate_signer_secret_lifecycle_policy(
        &markers,
        &[
            FALLBACK_SECRET_VIOLATION_REASON,
            "checkpoint_failed_signer_secret_contract",
        ],
    )
    .expect_err("forbidden fallback signer reason code must fail closed");

    assert!(
        error.contains("forbidden_reason_code:fallback_signer_secret_present_violation"),
        "error must preserve deterministic fallback signer violation reason code: {error}"
    );
}

#[test]
fn policy_checker_rejects_missing_required_lifecycle_markers() {
    let mut markers = complete_policy_markers();
    markers.remove("signer_secret_lifecycle_policy_contract_status");

    let error = evaluate_signer_secret_lifecycle_policy(&markers, &[])
        .expect_err("missing lifecycle markers must fail closed");
    assert!(
        error.contains("missing_required_marker:signer_secret_lifecycle_policy_contract_status"),
        "error must preserve deterministic missing-marker reason: {error}"
    );
}

#[test]
fn policy_checker_accepts_complete_marker_set_without_fallback_violation() {
    let markers = complete_policy_markers();
    let result = evaluate_signer_secret_lifecycle_policy(
        &markers,
        &["checkpoint_failed_signer_quorum_contract"],
    );

    assert!(
        result.is_ok(),
        "complete marker set without forbidden fallback reason must pass policy checks: {result:?}"
    );
}

#[test]
fn docs_declare_signer_secret_lifecycle_policy_markers_and_closure_chain() {
    assert!(
        CI_STRATEGY_DOC.contains("### Signer Secret Lifecycle Policy Contract"),
        "ci strategy docs must declare signer secret lifecycle policy section"
    );
    for marker in REQUIRED_CI_DOC_MARKERS {
        assert!(
            CI_STRATEGY_DOC.contains(marker),
            "ci strategy docs missing signer secret lifecycle marker: {marker}"
        );
    }
    assert!(
        CI_STRATEGY_DOC.contains(
            "cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract -- --nocapture"
        ),
        "ci strategy docs must declare signer secret lifecycle contract test command"
    );

    assert!(
        PRODUCTION_NEXT_STEPS_DOC.contains("### R26.2 Signer Secret-Lifecycle Contract Closure"),
        "production next-steps docs must declare signer secret lifecycle closure section"
    );
    for marker in REQUIRED_PLAN_DOC_MARKERS {
        assert!(
            PRODUCTION_NEXT_STEPS_DOC.contains(marker),
            "production next-steps docs missing signer secret lifecycle closure marker: {marker}"
        );
    }

    let extracted = extract_policy_markers(CI_STRATEGY_DOC);
    let evaluation = evaluate_signer_secret_lifecycle_policy(&extracted, &[]);
    assert!(
        evaluation.is_ok(),
        "docs-declared marker set must satisfy lifecycle policy checker: {evaluation:?}"
    );
}

fn complete_policy_markers() -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    markers.insert(
        "signer_secret_redaction_regression_guard_status".to_owned(),
        "active".to_owned(),
    );
    markers.insert(
        "signer_secret_redaction_policy".to_owned(),
        "raw_private_key_value_never_emitted".to_owned(),
    );
    markers.insert(
        "signer_secret_lifecycle_policy_contract_status".to_owned(),
        "active".to_owned(),
    );
    markers.insert(
        "signer_secret_lifecycle_policy_contract_version".to_owned(),
        "v1".to_owned(),
    );
    markers.insert(
        "signer_secret_lifecycle_forbidden_reason_code".to_owned(),
        FALLBACK_SECRET_VIOLATION_REASON.to_owned(),
    );
    markers.insert(
        "signer_secret_lifecycle_required_markers_csv".to_owned(),
        REQUIRED_LIFECYCLE_CSV_ENTRIES.join(","),
    );
    markers
}

fn extract_policy_markers(document: &str) -> BTreeMap<String, String> {
    let mut markers = BTreeMap::new();
    for key in REQUIRED_POLICY_MARKER_KEYS {
        if let Some(value) = extract_marker_value(document, key) {
            markers.insert((*key).to_owned(), value);
        }
    }
    markers
}

fn extract_marker_value(document: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=");
    let marker_index = document.find(&marker)?;
    let remainder = &document[(marker_index + marker.len())..];
    let value = remainder
        .split(['`', '\n', '\r'])
        .next()
        .unwrap_or("")
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn evaluate_signer_secret_lifecycle_policy(
    markers: &BTreeMap<String, String>,
    reason_codes: &[&str],
) -> Result<(), String> {
    if reason_codes.contains(&FALLBACK_SECRET_VIOLATION_REASON) {
        return Err(format!(
            "forbidden_reason_code:{FALLBACK_SECRET_VIOLATION_REASON}"
        ));
    }

    for key in REQUIRED_POLICY_MARKER_KEYS {
        let value = markers
            .get(*key)
            .ok_or_else(|| format!("missing_required_marker:{key}"))?;
        if value.trim().is_empty() {
            return Err(format!("empty_required_marker:{key}"));
        }
    }

    let csv = markers
        .get("signer_secret_lifecycle_required_markers_csv")
        .expect("required marker key checked above");
    let entries: Vec<&str> = csv.split(',').map(str::trim).collect();
    for required_entry in REQUIRED_LIFECYCLE_CSV_ENTRIES {
        if !entries.iter().any(|entry| entry == required_entry) {
            return Err(format!(
                "required_markers_csv_missing_entry:{required_entry}"
            ));
        }
    }

    let forbidden_reason_value = markers
        .get("signer_secret_lifecycle_forbidden_reason_code")
        .expect("required marker key checked above");
    if forbidden_reason_value != FALLBACK_SECRET_VIOLATION_REASON {
        return Err(format!(
            "forbidden_reason_code_mismatch:{forbidden_reason_value}"
        ));
    }

    Ok(())
}
