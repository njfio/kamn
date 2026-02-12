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

    for module in ["agent_key_hierarchy"] {
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
fn graduated_wave_six_marketplace_module_must_not_return_to_allowlist() {
    // Regression: #1985
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);

    for module in ["service_marketplace"] {
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
