use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const REPORT_SCHEMA_VERSION: &str = "kamn.core.public-api-surface-report.v1";
const BASELINE_SCHEMA_VERSION: &str = "kamn.core.public-api-surface-baseline.v1";
const THRESHOLD_SCHEMA_VERSION: &str = "kamn.core.public-api-surface-thresholds.v1";
const WAIVER_SCHEMA_VERSION: &str = "kamn.core.public-api-surface-waiver.v1";
const REASON_TAXONOMY_VERSION: &str = "kamn.core.public-api-surface-reason-taxonomy.v1";
const REASON_CODES_CSV: &str = "baseline_fixture_missing,baseline_fixture_invalid,baseline_schema_mismatch,baseline_threshold_missing,baseline_threshold_invalid,baseline_module_missing,module_source_missing,threshold_fixture_missing,threshold_fixture_invalid,threshold_schema_mismatch,threshold_value_invalid,waiver_fixture_invalid,waiver_schema_mismatch,waiver_missing_mitigation_issue,waiver_invalid_mitigation_issue,waiver_cap_exceeded,public_api_surface_fail_threshold_exceeded_unwaived,report_output_write_failed";

#[derive(Clone, Debug, Eq, PartialEq)]
struct ModuleSurface {
    module: String,
    public_items: usize,
    baseline_public_items: usize,
    delta_public_items: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ApiSurfaceReport {
    total_public_items: usize,
    baseline_total_public_items: usize,
    public_items_delta: i64,
    modules: Vec<ModuleSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PolicyThresholds {
    warn_total_delta_max: i64,
    fail_total_delta_max: i64,
    waiver_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PolicyWaiver {
    mitigation_issue: String,
    max_total_delta: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum PolicyStatus {
    Within,
    Warn,
    ExceptionApplied,
}

impl PolicyStatus {
    fn as_marker(&self) -> &'static str {
        match self {
            Self::Within => "within",
            Self::Warn => "warn",
            Self::ExceptionApplied => "exception-applied",
        }
    }
}

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn fail(reason_code: &str, detail: &str) -> ! {
    panic!(
        "reason_taxonomy_version={} reason_codes_csv={} reason_code={} detail={}",
        REASON_TAXONOMY_VERSION, REASON_CODES_CSV, reason_code, detail
    );
}

fn read_file(path: &Path, reason_code: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| fail(reason_code, &format!("{}: {}", path.display(), error)))
}

fn parse_key_value_fixture(raw: &str, reason_code: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = trimmed.split_once('=').unwrap_or_else(|| {
            fail(
                reason_code,
                &format!("line {} missing key=value form", index + 1),
            )
        });
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() {
            fail(reason_code, &format!("line {} has empty key", index + 1));
        }
        map.insert(key.to_owned(), value.to_owned());
    }
    map
}

fn required_value<'a>(map: &'a BTreeMap<String, String>, key: &str, reason_code: &str) -> &'a str {
    map.get(key)
        .map(String::as_str)
        .unwrap_or_else(|| fail(reason_code, &format!("missing required key {}", key)))
}

fn required_i64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> i64 {
    let value = required_value(map, key, reason_code);
    value.parse::<i64>().unwrap_or_else(|error| {
        fail(
            reason_code,
            &format!("key {} must parse as integer: {}", key, error),
        )
    })
}

fn parse_public_modules(lib_rs: &str) -> Vec<String> {
    let mut modules = Vec::new();
    for line in lib_rs.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            modules.push(rest.trim_end_matches(';').to_owned());
        }
    }
    modules.sort();
    modules
}

fn gather_rs_paths(dir: &Path, output: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|error| {
        fail(
            "module_source_missing",
            &format!("failed to read {}: {}", dir.display(), error),
        )
    });
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| {
                fail(
                    "module_source_missing",
                    &format!("failed to read dir entry in {}: {}", dir.display(), error),
                )
            })
            .path();
        if path.is_dir() {
            gather_rs_paths(&path, output);
            continue;
        }
        if path.extension().is_some_and(|ext| ext == "rs") {
            output.push(path);
        }
    }
}

fn module_source_paths(module: &str) -> Vec<PathBuf> {
    let src_root = repo_path("src");
    let mut paths = Vec::new();

    let module_root = src_root.join(format!("{}.rs", module));
    if module_root.is_file() {
        paths.push(module_root);
    }

    let module_dir = src_root.join(module);
    if module_dir.is_dir() {
        gather_rs_paths(&module_dir, &mut paths);
    }

    paths.sort();
    paths.dedup();
    paths
}

fn is_public_api_item_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub(super)")
        || trimmed.starts_with("pub(in ")
        || trimmed.starts_with("pub(in\t")
    {
        return false;
    }

    let mut tokens = trimmed.split_whitespace();
    if tokens.next() != Some("pub") {
        return false;
    }
    match tokens.next() {
        Some("fn" | "struct" | "enum" | "trait" | "type" | "const" | "static" | "mod" | "use") => {
            true
        }
        Some("async" | "unsafe") => tokens.next() == Some("fn"),
        Some("extern") => true,
        _ => false,
    }
}

fn count_public_items(path: &Path) -> usize {
    let content = read_file(path, "module_source_missing");
    content
        .lines()
        .filter(|line| is_public_api_item_line(line))
        .count()
}

fn load_baseline(modules: &[String]) -> (usize, BTreeMap<String, usize>) {
    let baseline_path = repo_path("../../fixtures/ci/kamn_core_public_api_surface_baseline.env");
    if !baseline_path.is_file() {
        fail(
            "baseline_fixture_missing",
            &format!("missing baseline fixture {}", baseline_path.display()),
        );
    }
    let baseline_raw = read_file(&baseline_path, "baseline_fixture_missing");
    let baseline_map = parse_key_value_fixture(&baseline_raw, "baseline_fixture_invalid");
    let schema_version =
        required_value(&baseline_map, "schema_version", "baseline_schema_mismatch");
    if schema_version != BASELINE_SCHEMA_VERSION {
        fail(
            "baseline_schema_mismatch",
            &format!(
                "unexpected schema {} in {}",
                schema_version,
                baseline_path.display()
            ),
        );
    }

    let baseline_total = required_i64(
        &baseline_map,
        "total_public_items",
        "baseline_threshold_invalid",
    );
    if baseline_total < 0 {
        fail(
            "baseline_threshold_invalid",
            "total_public_items must be non-negative",
        );
    }

    let baseline_module_count =
        required_i64(&baseline_map, "module_count", "baseline_threshold_invalid");
    if baseline_module_count < 0 {
        fail(
            "baseline_threshold_invalid",
            "module_count must be non-negative",
        );
    }
    if baseline_module_count as usize != modules.len() {
        fail(
            "baseline_threshold_invalid",
            &format!(
                "module_count mismatch: fixture={} actual={}",
                baseline_module_count,
                modules.len()
            ),
        );
    }

    let mut module_baselines = BTreeMap::new();
    for module in modules {
        let key = format!("module_public_items.{}", module);
        let count = required_i64(&baseline_map, key.as_str(), "baseline_module_missing");
        if count < 0 {
            fail(
                "baseline_threshold_invalid",
                &format!("{} must be non-negative", key),
            );
        }
        module_baselines.insert(module.clone(), count as usize);
    }

    (baseline_total as usize, module_baselines)
}

fn load_thresholds() -> PolicyThresholds {
    let threshold_path = repo_path("../../.ci/kamn-core-public-api-surface-thresholds.env");
    if !threshold_path.is_file() {
        fail(
            "threshold_fixture_missing",
            &format!("missing threshold fixture {}", threshold_path.display()),
        );
    }
    let raw = read_file(&threshold_path, "threshold_fixture_missing");
    let map = parse_key_value_fixture(&raw, "threshold_fixture_invalid");
    let schema_version = required_value(&map, "schema_version", "threshold_schema_mismatch");
    if schema_version != THRESHOLD_SCHEMA_VERSION {
        fail(
            "threshold_schema_mismatch",
            &format!(
                "unexpected schema {} in {}",
                schema_version,
                threshold_path.display()
            ),
        );
    }

    let warn_total_delta_max =
        required_i64(&map, "warn_total_delta_max", "threshold_value_invalid");
    let fail_total_delta_max =
        required_i64(&map, "fail_total_delta_max", "threshold_value_invalid");
    if warn_total_delta_max > fail_total_delta_max {
        fail(
            "threshold_value_invalid",
            &format!(
                "warn_total_delta_max ({}) must be <= fail_total_delta_max ({})",
                warn_total_delta_max, fail_total_delta_max
            ),
        );
    }

    let waiver_file = map
        .get("waiver_file")
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "None")
        .map(repo_path);

    PolicyThresholds {
        warn_total_delta_max,
        fail_total_delta_max,
        waiver_file,
    }
}

fn load_waiver(path: &Path) -> PolicyWaiver {
    let raw = read_file(path, "waiver_fixture_invalid");
    let map = parse_key_value_fixture(&raw, "waiver_fixture_invalid");
    let schema_version = required_value(&map, "schema_version", "waiver_schema_mismatch");
    if schema_version != WAIVER_SCHEMA_VERSION {
        fail(
            "waiver_schema_mismatch",
            &format!(
                "unexpected waiver schema {} in {}",
                schema_version,
                path.display()
            ),
        );
    }
    let mitigation_issue =
        required_value(&map, "mitigation_issue", "waiver_missing_mitigation_issue");
    if !mitigation_issue.starts_with('#')
        || mitigation_issue
            .trim_start_matches('#')
            .chars()
            .any(|ch| !ch.is_ascii_digit())
    {
        fail(
            "waiver_invalid_mitigation_issue",
            &format!(
                "mitigation_issue must be #<digits>, got {} in {}",
                mitigation_issue,
                path.display()
            ),
        );
    }
    let max_total_delta = required_i64(&map, "max_total_delta", "waiver_fixture_invalid");
    if max_total_delta < 0 {
        fail(
            "waiver_fixture_invalid",
            &format!("max_total_delta must be non-negative in {}", path.display()),
        );
    }

    PolicyWaiver {
        mitigation_issue: mitigation_issue.to_owned(),
        max_total_delta,
    }
}

fn build_report(
    modules: &[String],
    baseline_total_public_items: usize,
    baseline_module_public_items: &BTreeMap<String, usize>,
) -> ApiSurfaceReport {
    let mut per_module = Vec::new();
    let mut total_public_items = 0usize;
    for module in modules {
        let source_paths = module_source_paths(module);
        if source_paths.is_empty() {
            fail(
                "module_source_missing",
                &format!("module {} has no source files", module),
            );
        }
        let module_public_items = source_paths
            .iter()
            .map(|path| count_public_items(path))
            .sum::<usize>();
        let baseline_public_items = *baseline_module_public_items
            .get(module)
            .unwrap_or_else(|| fail("baseline_module_missing", module));
        total_public_items += module_public_items;
        per_module.push(ModuleSurface {
            module: module.clone(),
            public_items: module_public_items,
            baseline_public_items,
            delta_public_items: module_public_items as i64 - baseline_public_items as i64,
        });
    }

    ApiSurfaceReport {
        total_public_items,
        baseline_total_public_items,
        public_items_delta: total_public_items as i64 - baseline_total_public_items as i64,
        modules: per_module,
    }
}

fn evaluate_policy(
    report: &ApiSurfaceReport,
    thresholds: &PolicyThresholds,
) -> (PolicyStatus, String) {
    if report.public_items_delta <= thresholds.warn_total_delta_max {
        return (PolicyStatus::Within, "none".to_owned());
    }
    if report.public_items_delta <= thresholds.fail_total_delta_max {
        return (
            PolicyStatus::Warn,
            "public_api_surface_warn_threshold_exceeded".to_owned(),
        );
    }

    if let Some(waiver_path) = &thresholds.waiver_file {
        if waiver_path.is_file() {
            let waiver = load_waiver(waiver_path);
            if report.public_items_delta <= waiver.max_total_delta {
                return (
                    PolicyStatus::ExceptionApplied,
                    format!(
                        "public_api_surface_fail_threshold_exceeded_waived:{}",
                        waiver.mitigation_issue
                    ),
                );
            }
            fail(
                "waiver_cap_exceeded",
                &format!(
                    "public_items_delta {} exceeds waiver cap {} ({})",
                    report.public_items_delta,
                    waiver.max_total_delta,
                    waiver_path.display()
                ),
            );
        }
    }

    fail(
        "public_api_surface_fail_threshold_exceeded_unwaived",
        &format!(
            "public_items_delta {} exceeds fail_total_delta_max {}",
            report.public_items_delta, thresholds.fail_total_delta_max
        ),
    );
}

fn render_report(
    report: &ApiSurfaceReport,
    thresholds: &PolicyThresholds,
    status: &PolicyStatus,
    reason_codes: &str,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("report_schema_version={}", REPORT_SCHEMA_VERSION));
    lines.push(format!(
        "policy_schema_version={}",
        THRESHOLD_SCHEMA_VERSION
    ));
    lines.push(format!(
        "reason_taxonomy_version={}",
        REASON_TAXONOMY_VERSION
    ));
    lines.push(format!("reason_codes={}", reason_codes));
    lines.push(format!("policy_status={}", status.as_marker()));
    lines.push(format!(
        "warn_total_delta_max={}",
        thresholds.warn_total_delta_max
    ));
    lines.push(format!(
        "fail_total_delta_max={}",
        thresholds.fail_total_delta_max
    ));
    lines.push(format!("total_public_items={}", report.total_public_items));
    lines.push(format!(
        "baseline_total_public_items={}",
        report.baseline_total_public_items
    ));
    lines.push(format!("public_items_delta={}", report.public_items_delta));
    lines.push(format!("module_count={}", report.modules.len()));
    for module in &report.modules {
        lines.push(format!(
            "module_public_items.{}={}",
            module.module, module.public_items
        ));
        lines.push(format!(
            "module_public_items_baseline.{}={}",
            module.module, module.baseline_public_items
        ));
        lines.push(format!(
            "module_public_items_delta.{}={}",
            module.module, module.delta_public_items
        ));
    }
    lines.join("\n") + "\n"
}

fn maybe_write_report(report: &str) {
    let output_path = match std::env::var("KAMN_CORE_PUBLIC_API_SURFACE_REPORT_OUTPUT") {
        Ok(path) if !path.trim().is_empty() => PathBuf::from(path),
        _ => return,
    };
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| {
            fail(
                "report_output_write_failed",
                &format!("failed to create {}: {}", parent.display(), error),
            )
        });
    }
    fs::write(&output_path, report).unwrap_or_else(|error| {
        fail(
            "report_output_write_failed",
            &format!("{}: {}", output_path.display(), error),
        )
    });
}

fn compute_report_with_policy() -> (
    ApiSurfaceReport,
    PolicyThresholds,
    PolicyStatus,
    String,
    String,
) {
    let lib_rs = read_file(&repo_path("src/lib.rs"), "module_source_missing");
    let modules = parse_public_modules(&lib_rs);
    let (baseline_total_public_items, baseline_module_public_items) = load_baseline(&modules);
    let thresholds = load_thresholds();
    let report = build_report(
        &modules,
        baseline_total_public_items,
        &baseline_module_public_items,
    );
    let (status, reason_codes) = evaluate_policy(&report, &thresholds);
    let rendered = render_report(&report, &thresholds, &status, &reason_codes);
    (report, thresholds, status, reason_codes, rendered)
}

#[test]
fn public_api_surface_report_schema_is_deterministic() {
    let (report, thresholds, status, reason_codes, rendered) = compute_report_with_policy();
    maybe_write_report(&rendered);

    assert!(report
        .modules
        .windows(2)
        .all(|pair| pair[0].module <= pair[1].module));
    assert_eq!(
        report.public_items_delta,
        report.total_public_items as i64 - report.baseline_total_public_items as i64
    );

    assert!(rendered.contains(&format!("report_schema_version={}", REPORT_SCHEMA_VERSION)));
    assert!(rendered.contains(&format!(
        "policy_schema_version={}",
        THRESHOLD_SCHEMA_VERSION
    )));
    assert!(rendered.contains(&format!("policy_status={}", status.as_marker())));
    assert!(rendered.contains(&format!("reason_codes={}", reason_codes)));
    assert!(rendered.contains(&format!(
        "warn_total_delta_max={}",
        thresholds.warn_total_delta_max
    )));
    assert!(rendered.contains(&format!(
        "fail_total_delta_max={}",
        thresholds.fail_total_delta_max
    )));
}

#[test]
fn public_api_surface_policy_enforces_warn_fail_contract() {
    let (report, thresholds, status, _reason_codes, _rendered) = compute_report_with_policy();
    assert!(matches!(
        status,
        PolicyStatus::Within | PolicyStatus::Warn | PolicyStatus::ExceptionApplied
    ));
    assert!(
        report.public_items_delta <= thresholds.fail_total_delta_max
            || matches!(status, PolicyStatus::ExceptionApplied),
        "delta={} fail_max={} status={}",
        report.public_items_delta,
        thresholds.fail_total_delta_max,
        status.as_marker()
    );
}
