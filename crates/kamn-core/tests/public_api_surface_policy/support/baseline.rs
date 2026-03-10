use crate::support::constants::BASELINE_SCHEMA_VERSION;
use crate::support::fixtures::{parse_key_value_fixture, required_i64, required_value};
use crate::support::paths::{fail, read_file};
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) fn load_baseline(path: &Path, modules: &[String]) -> (usize, BTreeMap<String, usize>) {
    if !path.is_file() {
        fail("baseline_fixture_missing", &format!("missing baseline fixture {}", path.display()));
    }
    let baseline_map = parse_key_value_fixture(&read_file(path, "baseline_fixture_missing"), "baseline_fixture_invalid");
    assert_baseline_schema(&baseline_map, path);
    let baseline_total = non_negative_value(&baseline_map, "total_public_items");
    assert_module_count(&baseline_map, modules.len());
    (baseline_total, module_baselines(modules, &baseline_map))
}

fn assert_baseline_schema(map: &BTreeMap<String, String>, path: &Path) {
    let schema_version = required_value(map, "schema_version", "baseline_schema_mismatch");
    if schema_version != BASELINE_SCHEMA_VERSION {
        fail(
            "baseline_schema_mismatch",
            &format!("unexpected schema {} in {}", schema_version, path.display()),
        );
    }
}

fn assert_module_count(map: &BTreeMap<String, String>, expected: usize) {
    let module_count = non_negative_value(map, "module_count");
    if module_count != expected {
        fail(
            "baseline_threshold_invalid",
            &format!("module_count mismatch: fixture={} actual={}", module_count, expected),
        );
    }
}

fn module_baselines(
    modules: &[String],
    map: &BTreeMap<String, String>,
) -> BTreeMap<String, usize> {
    modules
        .iter()
        .map(|module| (module.clone(), module_baseline(module, map)))
        .collect()
}

fn module_baseline(module: &str, map: &BTreeMap<String, String>) -> usize {
    let key = format!("module_public_items.{}", module);
    non_negative_reason(map, key.as_str(), "baseline_module_missing")
}

fn non_negative_value(map: &BTreeMap<String, String>, key: &str) -> usize {
    non_negative_reason(map, key, "baseline_threshold_invalid")
}

fn non_negative_reason(map: &BTreeMap<String, String>, key: &str, reason: &str) -> usize {
    let value = required_i64(map, key, reason);
    if value < 0 {
        fail(reason, &format!("{} must be non-negative", key));
    }
    value as usize
}
