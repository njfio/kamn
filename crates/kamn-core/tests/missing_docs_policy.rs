const CORE_LIB: &str = include_str!("../src/lib.rs");
const ALLOWLIST_FIXTURE: &str =
    include_str!("../../../fixtures/ci/kamn_core_missing_docs_allowlist.txt");
const GRADUATED_MODULES_FIXTURE: &str =
    include_str!("../../../fixtures/ci/kamn_core_missing_docs_graduated_modules.txt");

fn allowlisted_modules_from_core_lib(source: &str) -> Vec<String> {
    let mut allow_pending = false;
    let mut modules = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed == "#[allow(missing_docs)]" {
            allow_pending = true;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if allow_pending {
                modules.push(rest.trim_end_matches(';').to_owned());
            }
            allow_pending = false;
            continue;
        }

        if !trimmed.is_empty() {
            allow_pending = false;
        }
    }

    modules.sort();
    modules
}

fn allowlisted_modules_from_fixture(source: &str) -> Vec<String> {
    let mut modules: Vec<String> = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect();
    modules.sort();
    modules
}

#[test]
fn kamn_core_declares_missing_docs_warning_policy() {
    assert!(CORE_LIB.contains("#![warn(missing_docs)]"));
    assert!(!CORE_LIB.contains("#![allow(missing_docs)]"));
}

#[test]
fn kamn_core_missing_docs_allowlist_matches_fixture() {
    // Regression: #896
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);

    assert_eq!(actual, expected);
}

#[test]
fn namespaces_module_must_not_return_to_missing_docs_allowlist() {
    // Regression: #1334
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);

    assert!(
        !actual.iter().any(|module| module == "namespaces"),
        "namespaces module must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|module| module == "namespaces"),
        "allowlist fixture must keep namespaces module removed"
    );
}

#[test]
fn graduated_wave_two_modules_must_not_return_to_missing_docs_allowlist() {
    // Regression: #1365
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);

    for module in ["bootstrap", "kolme_runtime_commit", "state"] {
        assert!(
            !actual.iter().any(|candidate| candidate == module),
            "{module} must stay graduated from #[allow(missing_docs)]"
        );
        assert!(
            !expected.iter().any(|candidate| candidate == module),
            "allowlist fixture must keep {module} removed"
        );
    }
}

#[test]
fn graduated_wave_three_task_lifecycle_module_must_not_return_to_missing_docs_allowlist() {
    // Regression: #1601
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);

    assert!(
        !actual.iter().any(|candidate| candidate == "task_lifecycle"),
        "task_lifecycle must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected
            .iter()
            .any(|candidate| candidate == "task_lifecycle"),
        "allowlist fixture must keep task_lifecycle removed"
    );
}

#[test]
fn graduated_wave_four_runtime_safety_modules_must_not_return_to_allowlist() {
    // Regression: #1828
    // Regression: #1981
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);

    for module in ["key_recovery", "migrations", "signature_profile", "smoke"] {
        assert!(
            !actual.iter().any(|candidate| candidate == module),
            "{module} must stay graduated from #[allow(missing_docs)]"
        );
        assert!(
            !expected.iter().any(|candidate| candidate == module),
            "allowlist fixture must keep {module} removed"
        );
    }
}

#[test]
fn graduated_wave_five_identity_module_must_not_return_to_allowlist() {
    // Regression: #1983
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "agent_key_hierarchy";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_six_marketplace_module_must_not_return_to_allowlist() {
    // Regression: #1985
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "service_marketplace";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_seven_task_payment_module_must_not_return_to_allowlist() {
    // Regression: #1987
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "task_payment";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_eight_reputation_signals_module_must_not_return_to_allowlist() {
    // Regression: #1989
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "reputation_signals";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_nine_telegram_bridge_module_must_not_return_to_allowlist() {
    // Regression: #1991
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "telegram_bridge";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_ten_operator_actions_module_must_not_return_to_allowlist() {
    // Regression: #1993
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "operator_actions";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_eleven_transaction_module_must_not_return_to_allowlist() {
    // Regression: #1995
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "transaction";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twelve_task_artifacts_module_must_not_return_to_allowlist() {
    // Regression: #1997
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "task_artifacts";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirteen_direct_message_crypto_module_must_not_return_to_allowlist() {
    // Regression: #1999
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "direct_message_crypto";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fourteen_operator_binding_module_must_not_return_to_allowlist() {
    // Regression: #2001
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "operator_binding";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifteen_anti_spam_module_must_not_return_to_allowlist() {
    // Regression: #2003
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "anti_spam";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_sixteen_discord_bridge_module_must_not_return_to_allowlist() {
    // Regression: #2005
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "discord_bridge";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_seventeen_key_lifecycle_module_must_not_return_to_allowlist() {
    // Regression: #2007
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "key_lifecycle";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_eighteen_validator_lifecycle_module_must_not_return_to_allowlist() {
    // Regression: #2009
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "validator_lifecycle";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_nineteen_group_channel_crypto_module_must_not_return_to_allowlist() {
    // Regression: #2011
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "group_channel_crypto";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_config_module_must_not_return_to_allowlist() {
    // Regression: #2013
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "config";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_one_content_storage_module_must_not_return_to_allowlist() {
    // Regression: #2015
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "content_storage";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_two_redaction_compliance_module_must_not_return_to_allowlist() {
    // Regression: #2017
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "redaction_compliance";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_three_cross_chain_bridge_module_must_not_return_to_allowlist() {
    // Regression: #2019
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "cross_chain_bridge";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_four_cross_chain_receipt_module_must_not_return_to_allowlist() {
    // Regression: #2021
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "cross_chain_receipt";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_five_content_lifecycle_module_must_not_return_to_allowlist() {
    // Regression: #2023
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "content_lifecycle";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_six_content_replication_module_must_not_return_to_allowlist() {
    // Regression: #2025
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "content_replication";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_seven_content_retrieval_module_must_not_return_to_allowlist() {
    // Regression: #2027
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "content_retrieval";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_eight_data_classification_module_must_not_return_to_allowlist() {
    // Regression: #2029
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "data_classification";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_twenty_nine_did_module_must_not_return_to_allowlist() {
    // Regression: #2031
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "did";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_did_registry_module_must_not_return_to_allowlist() {
    // Regression: #2033
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "did_registry";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_one_durable_guard_store_module_must_not_return_to_allowlist() {
    // Regression: #2035
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "durable_guard_store";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_two_audit_exports_module_must_not_return_to_allowlist() {
    // Regression: #2037
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "audit_exports";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_three_performance_targets_module_must_not_return_to_allowlist() {
    // Regression: #2039
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "performance_targets";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_four_retention_engine_module_must_not_return_to_allowlist() {
    // Regression: #2041
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "retention_engine";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_five_token_module_must_not_return_to_allowlist() {
    // Regression: #2043
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "token";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_six_observability_module_must_not_return_to_allowlist() {
    // Regression: #2045
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "observability";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_seven_operator_dashboard_api_module_must_not_return_to_allowlist() {
    // Regression: #2047
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "operator_dashboard_api";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_eight_operator_dashboard_ui_module_must_not_return_to_allowlist() {
    // Regression: #2049
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "operator_dashboard_ui";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_thirty_nine_invariants_module_must_not_return_to_allowlist() {
    // Regression: #2051
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "invariants";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_message_delivery_guards_module_must_not_return_to_allowlist() {
    // Regression: #2053
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "message_delivery_guards";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_one_trust_score_module_must_not_return_to_allowlist() {
    // Regression: #2055
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "trust_score";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_two_watchdog_module_must_not_return_to_allowlist() {
    // Regression: #2057
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "watchdog";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_three_message_envelope_module_must_not_return_to_allowlist() {
    // Regression: #2059
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "message_envelope";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_four_upgrade_orchestration_module_must_not_return_to_allowlist() {
    // Regression: #2061
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "upgrade_orchestration";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_five_escrow_module_must_not_return_to_allowlist() {
    // Regression: #2063
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "escrow";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_six_instruction_verify_module_must_not_return_to_allowlist() {
    // Regression: #2065
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "instruction_verify";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_seven_reputation_state_module_must_not_return_to_allowlist() {
    // Regression: #2067
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "reputation_state";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_eight_channel_policies_module_must_not_return_to_allowlist() {
    // Regression: #2069
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "channel_policies";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_forty_nine_bridge_adapter_module_must_not_return_to_allowlist() {
    // Regression: #2071
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "bridge_adapter";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_channel_models_module_must_not_return_to_allowlist() {
    // Regression: #2073
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "channel_models";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_one_governance_workflow_module_must_not_return_to_allowlist() {
    // Regression: #2075
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "governance_workflow";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_two_agent_upgrade_workflow_module_must_not_return_to_allowlist() {
    // Regression: #2077
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "agent_upgrade_workflow";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_three_signer_backend_module_must_not_return_to_allowlist() {
    // Regression: #2079
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "signer_backend";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_four_task_operations_module_must_not_return_to_allowlist() {
    // Regression: #2081
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "task_operations";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_five_zk_message_proofs_module_must_not_return_to_allowlist() {
    // Regression: #2083
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "zk_message_proofs";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_six_message_lifecycle_module_must_not_return_to_allowlist() {
    // Regression: #2085
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "message_lifecycle";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_wave_fifty_seven_runtime_module_must_not_return_to_allowlist() {
    // Regression: #2087
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let module = "runtime";

    assert!(
        !actual.iter().any(|candidate| candidate == module),
        "{module} must stay graduated from #[allow(missing_docs)]"
    );
    assert!(
        !expected.iter().any(|candidate| candidate == module),
        "allowlist fixture must keep {module} removed"
    );
}

#[test]
fn graduated_modules_fixture_must_not_overlap_missing_docs_allowlist() {
    // Regression: #1723
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let graduated = allowlisted_modules_from_fixture(GRADUATED_MODULES_FIXTURE);

    for module in graduated {
        assert!(
            !actual.iter().any(|candidate| candidate == &module),
            "{module} must stay graduated from #[allow(missing_docs)]"
        );
        assert!(
            !expected.iter().any(|candidate| candidate == &module),
            "allowlist fixture must keep {module} removed"
        );
    }
}
