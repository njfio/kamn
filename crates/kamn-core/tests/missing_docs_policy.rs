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

fn assert_modules_absent(actual: &[String], expected: &[String], modules: &[String]) {
    for module in modules {
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
fn graduated_modules_fixture_must_not_overlap_missing_docs_allowlist() {
    // Consolidated regression suite for prior wave-by-wave graduation checks.
    // Regression: #1334, #1365, #1601, #1828, #1981, #1983-#2087
    let actual = allowlisted_modules_from_core_lib(CORE_LIB);
    let expected = allowlisted_modules_from_fixture(ALLOWLIST_FIXTURE);
    let graduated = allowlisted_modules_from_fixture(GRADUATED_MODULES_FIXTURE);

    assert_modules_absent(&actual, &expected, &graduated);
}

#[test]
fn graduated_modules_fixture_tracks_expected_minimum_surface() {
    // Prevent accidental fixture truncation that would silently reduce graduation coverage.
    let graduated = allowlisted_modules_from_fixture(GRADUATED_MODULES_FIXTURE);
    assert!(
        graduated.len() >= 60,
        "expected at least 60 graduated modules in fixture, found {}",
        graduated.len()
    );
}
