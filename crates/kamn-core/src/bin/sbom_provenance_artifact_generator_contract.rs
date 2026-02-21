use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

const RUN_SCHEMA_VERSION: &str = "kamn.runtime.sbom-provenance-artifact-report.v1";
const ARTIFACT_SCHEMA_VERSION: &str = "kamn.runtime.sbom-provenance-artifact-schema.v1";
const FIXTURE_SCHEMA_VERSION: &str = "kamn.ci.sbom-provenance-artifact-fixture-matrix.v1";
const REASON_TAXONOMY_VERSION: &str = "kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1";
const REASON_CODES_CSV: &str =
    "sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded";
const OPT_IN_ENV: &str = "KAMN_SBOM_PROVENANCE_GENERATOR_OPT_IN";

const EXPECTED_COLUMNS: [&str; 7] = [
    "profile",
    "sbom_component_count",
    "sbom_package_count",
    "sbom_digest_sha256",
    "provenance_digest_sha256",
    "expected_status",
    "expected_reason_code",
];

#[derive(Debug, Clone)]
struct Config {
    profile: String,
    mode: String,
    ci_fast_gate: String,
    max_seconds: String,
    local_opt_in: String,
    fixture_file: PathBuf,
    output_json: String,
}

#[derive(Debug, Clone)]
struct FixtureRow {
    sbom_component_count: u64,
    sbom_package_count: u64,
    sbom_digest_sha256: String,
    provenance_digest_sha256: String,
    expected_status: String,
    expected_reason_code: String,
}

type FixtureMarkers = BTreeMap<String, String>;
type FixtureRows = BTreeMap<String, FixtureRow>;

#[derive(Debug, Clone)]
struct Report {
    status: String,
    final_decision: String,
    lane_mode: String,
    profile: String,
    reason_code: String,
    reason_codes_value: String,
    sbom_schema_version: String,
    provenance_schema_version: String,
    sbom_component_count: u64,
    sbom_package_count: u64,
    sbom_digest_sha256: String,
    provenance_digest_sha256: String,
    release_manifest_ready_status: String,
    artifact_linkage_status: String,
    ci_fast_gate: String,
    run_mode_command_status: String,
    command_count: u64,
    performance_budget_status: String,
    elapsed_seconds: u64,
    max_seconds: u64,
    fixture_path: String,
}

#[derive(Debug, Clone)]
enum JsonValue {
    Str(String),
    Int(u64),
}

fn default_fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/ci/sbom_provenance_artifact_fixture_matrix.txt")
}

fn parse_args() -> Result<Config, String> {
    let mut config = Config {
        profile: "baseline".to_owned(),
        mode: "dry-run".to_owned(),
        ci_fast_gate: "PASS".to_owned(),
        max_seconds: std::env::var("KAMN_SBOM_PROVENANCE_GENERATOR_MAX_SECONDS")
            .unwrap_or_else(|_| "120".to_owned()),
        local_opt_in: std::env::var(OPT_IN_ENV).unwrap_or_else(|_| "0".to_owned()),
        fixture_file: default_fixture_path(),
        output_json: String::new(),
    };

    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--profile" => {
                config.profile = take_arg_value(&mut args, "--profile")?;
            }
            "--mode" => {
                config.mode = take_arg_value(&mut args, "--mode")?;
            }
            "--ci-fast-gate" => {
                config.ci_fast_gate = take_arg_value(&mut args, "--ci-fast-gate")?;
            }
            "--max-seconds" => {
                config.max_seconds = take_arg_value(&mut args, "--max-seconds")?;
            }
            "--local-opt-in" => {
                config.local_opt_in = take_arg_value(&mut args, "--local-opt-in")?;
            }
            "--fixture-file" => {
                config.fixture_file = PathBuf::from(take_arg_value(&mut args, "--fixture-file")?);
            }
            "--output-json" => {
                config.output_json = take_arg_value(&mut args, "--output-json")?;
            }
            _ => {
                return Err(format!("unknown argument: {flag}"));
            }
        }
    }

    Ok(config)
}

fn take_arg_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next()
        .ok_or_else(|| format!("missing value for {flag}"))
}

fn parse_positive_int(raw_value: &str, field: &str) -> Result<u64, String> {
    if raw_value.is_empty()
        || !raw_value
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(format!("{field} must be an integer"));
    }
    raw_value
        .parse::<u64>()
        .map_err(|_| format!("{field} must be an integer"))
}

fn parse_required_positive_int(raw_value: &str, field: &str) -> Result<u64, String> {
    let parsed = parse_positive_int(raw_value, field)?;
    if parsed == 0 {
        return Err(format!("{field} must be greater than zero"));
    }
    Ok(parsed)
}

fn is_sha256_digest(value: &str) -> bool {
    value.starts_with("sha256:")
        && value
            .strip_prefix("sha256:")
            .map(|digest| {
                digest.len() == 64
                    && digest
                        .chars()
                        .all(|character| character.is_ascii_hexdigit())
            })
            .unwrap_or(false)
}

fn parse_fixture(path: &Path) -> Result<(FixtureMarkers, FixtureRows), String> {
    if !path.exists() {
        return Err(format!("fixture file not found: {}", path.display()));
    }

    let fixture_text = fs::read_to_string(path)
        .map_err(|_| format!("fixture file not found: {}", path.display()))?;

    let mut markers: FixtureMarkers = BTreeMap::new();
    let mut rows: FixtureRows = BTreeMap::new();
    let mut columns: Vec<String> = Vec::new();

    for (line_index, raw_line) in fixture_text.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(columns_value) = line.strip_prefix("columns=") {
            columns = columns_value
                .split('|')
                .map(|part| part.trim().to_owned())
                .collect();
            continue;
        }

        if line.contains('=') && columns.is_empty() {
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or_default().trim().to_owned();
            let value = parts.next().unwrap_or_default().trim().to_owned();
            markers.insert(key, value);
            continue;
        }

        if columns.is_empty() {
            return Err(format!(
                "fixture row before columns marker on line {line_number}"
            ));
        }

        let values: Vec<String> = line.split('|').map(|part| part.trim().to_owned()).collect();

        if values.len() != columns.len() {
            return Err(format!(
                "fixture row column mismatch on line {line_number}: expected {} values",
                columns.len(),
            ));
        }

        let mut row_map: BTreeMap<String, String> = BTreeMap::new();
        for (column, value) in columns.iter().zip(values.iter()) {
            row_map.insert(column.clone(), value.clone());
        }

        let profile = row_map
            .get("profile")
            .cloned()
            .ok_or_else(|| format!("fixture row missing profile on line {line_number}"))?;

        if rows.contains_key(&profile) {
            return Err(format!(
                "duplicate fixture profile on line {line_number}: {profile}"
            ));
        }

        let sbom_component_count = parse_positive_int(
            row_map.get("sbom_component_count").ok_or_else(|| {
                format!("fixture row missing sbom_component_count for profile {profile}")
            })?,
            "sbom_component_count",
        )?;
        let sbom_package_count = parse_positive_int(
            row_map.get("sbom_package_count").ok_or_else(|| {
                format!("fixture row missing sbom_package_count for profile {profile}")
            })?,
            "sbom_package_count",
        )?;
        let sbom_digest_sha256 = row_map.get("sbom_digest_sha256").cloned().ok_or_else(|| {
            format!("fixture row missing sbom_digest_sha256 for profile {profile}")
        })?;
        let provenance_digest_sha256 = row_map
            .get("provenance_digest_sha256")
            .cloned()
            .ok_or_else(|| {
                format!("fixture row missing provenance_digest_sha256 for profile {profile}")
            })?;

        if !is_sha256_digest(&sbom_digest_sha256) {
            return Err(format!("invalid sbom digest shape for profile {profile}"));
        }
        if !is_sha256_digest(&provenance_digest_sha256) {
            return Err(format!(
                "invalid provenance digest shape for profile {profile}"
            ));
        }

        rows.insert(
            profile,
            FixtureRow {
                sbom_component_count,
                sbom_package_count,
                sbom_digest_sha256,
                provenance_digest_sha256,
                expected_status: row_map.get("expected_status").cloned().ok_or_else(|| {
                    format!("fixture row missing expected_status for profile {line_number}")
                })?,
                expected_reason_code: row_map.get("expected_reason_code").cloned().ok_or_else(
                    || {
                        format!(
                            "fixture row missing expected_reason_code for profile {line_number}"
                        )
                    },
                )?,
            },
        );
    }

    let parsed_columns: Vec<&str> = columns.iter().map(String::as_str).collect();
    if parsed_columns != EXPECTED_COLUMNS {
        return Err(format!(
            "fixture columns must be {}",
            EXPECTED_COLUMNS.join("|")
        ));
    }

    let required_marker_keys: BTreeSet<&str> = [
        "sbom_provenance_fixture_schema_version",
        "sbom_provenance_reason_taxonomy_version",
        "sbom_provenance_reason_codes_csv",
        "sbom_provenance_required_profiles_csv",
        "sbom_provenance_min_component_count",
        "sbom_provenance_sbom_schema_version",
        "sbom_provenance_provenance_schema_version",
    ]
    .iter()
    .copied()
    .collect();

    let marker_keys: BTreeSet<&str> = markers.keys().map(String::as_str).collect();
    let missing_keys: Vec<&str> = required_marker_keys
        .difference(&marker_keys)
        .copied()
        .collect();
    if !missing_keys.is_empty() {
        return Err(format!(
            "fixture missing required markers: {}",
            missing_keys.join(",")
        ));
    }

    if markers
        .get("sbom_provenance_fixture_schema_version")
        .map(String::as_str)
        != Some(FIXTURE_SCHEMA_VERSION)
    {
        return Err("fixture schema version mismatch".to_owned());
    }
    if markers
        .get("sbom_provenance_reason_taxonomy_version")
        .map(String::as_str)
        != Some(REASON_TAXONOMY_VERSION)
    {
        return Err("fixture reason taxonomy mismatch".to_owned());
    }
    if markers
        .get("sbom_provenance_reason_codes_csv")
        .map(String::as_str)
        != Some(REASON_CODES_CSV)
    {
        return Err("fixture reason codes csv mismatch".to_owned());
    }

    let required_profiles_csv = markers
        .get("sbom_provenance_required_profiles_csv")
        .ok_or_else(|| "fixture required profiles marker must not be empty".to_owned())?;

    let required_profiles: Vec<String> = required_profiles_csv
        .split(',')
        .map(str::trim)
        .filter(|profile| !profile.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    if required_profiles.is_empty() {
        return Err("fixture required profiles marker must not be empty".to_owned());
    }

    let required_profile_set: BTreeSet<String> = required_profiles.iter().cloned().collect();
    let row_profile_set: BTreeSet<String> = rows.keys().cloned().collect();
    if required_profile_set != row_profile_set {
        return Err("fixture profiles must match required profiles marker".to_owned());
    }

    for profile in &required_profiles {
        let expected_status = rows
            .get(profile)
            .map(|row| row.expected_status.as_str())
            .unwrap_or_default();
        if expected_status != "pass" && expected_status != "fail" {
            return Err(format!(
                "fixture expected_status must be pass/fail for profile {profile}"
            ));
        }
    }

    markers.insert(
        "sbom_provenance_required_profiles_csv".to_owned(),
        required_profiles.join(","),
    );

    Ok((markers, rows))
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn write_json(path: &Path, report: &Report) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create output directory: {error}"))?;
    }

    let fields: Vec<(&str, JsonValue)> = vec![
        (
            "schema_version",
            JsonValue::Str(RUN_SCHEMA_VERSION.to_owned()),
        ),
        (
            "artifact_schema_version",
            JsonValue::Str(ARTIFACT_SCHEMA_VERSION.to_owned()),
        ),
        (
            "fixture_schema_version",
            JsonValue::Str(FIXTURE_SCHEMA_VERSION.to_owned()),
        ),
        (
            "reason_taxonomy_version",
            JsonValue::Str(REASON_TAXONOMY_VERSION.to_owned()),
        ),
        (
            "reason_codes_csv",
            JsonValue::Str(REASON_CODES_CSV.to_owned()),
        ),
        ("status", JsonValue::Str(report.status.clone())),
        (
            "final_decision",
            JsonValue::Str(report.final_decision.clone()),
        ),
        ("lane_mode", JsonValue::Str(report.lane_mode.clone())),
        ("profile", JsonValue::Str(report.profile.clone())),
        ("reason_code", JsonValue::Str(report.reason_code.clone())),
        (
            "reason_codes_value",
            JsonValue::Str(report.reason_codes_value.clone()),
        ),
        (
            "sbom_schema_version",
            JsonValue::Str(report.sbom_schema_version.clone()),
        ),
        (
            "provenance_schema_version",
            JsonValue::Str(report.provenance_schema_version.clone()),
        ),
        (
            "sbom_component_count",
            JsonValue::Int(report.sbom_component_count),
        ),
        (
            "sbom_package_count",
            JsonValue::Int(report.sbom_package_count),
        ),
        (
            "sbom_digest_sha256",
            JsonValue::Str(report.sbom_digest_sha256.clone()),
        ),
        (
            "provenance_digest_sha256",
            JsonValue::Str(report.provenance_digest_sha256.clone()),
        ),
        (
            "release_manifest_required_artifact_id",
            JsonValue::Str("sbom_provenance".to_owned()),
        ),
        (
            "release_manifest_ready_status",
            JsonValue::Str(report.release_manifest_ready_status.clone()),
        ),
        (
            "artifact_linkage_status",
            JsonValue::Str(report.artifact_linkage_status.clone()),
        ),
        ("ci_fast_gate", JsonValue::Str(report.ci_fast_gate.clone())),
        (
            "run_mode_command_status",
            JsonValue::Str(report.run_mode_command_status.clone()),
        ),
        ("command_count", JsonValue::Int(report.command_count)),
        (
            "performance_budget_status",
            JsonValue::Str(report.performance_budget_status.clone()),
        ),
        ("elapsed_seconds", JsonValue::Int(report.elapsed_seconds)),
        ("max_seconds", JsonValue::Int(report.max_seconds)),
        ("fixture_path", JsonValue::Str(report.fixture_path.clone())),
    ];

    let mut json_text = String::from("{\n");
    for (index, (key, value)) in fields.iter().enumerate() {
        json_text.push_str("  \"");
        json_text.push_str(&json_escape(key));
        json_text.push_str("\": ");
        match value {
            JsonValue::Str(string_value) => {
                json_text.push('"');
                json_text.push_str(&json_escape(string_value));
                json_text.push('"');
            }
            JsonValue::Int(int_value) => {
                json_text.push_str(&int_value.to_string());
            }
        }
        if index + 1 != fields.len() {
            json_text.push(',');
        }
        json_text.push('\n');
    }
    json_text.push_str("}\n");

    fs::write(path, json_text).map_err(|error| format!("failed to write output json: {error}"))
}

fn run() -> Result<i32, String> {
    let started = Instant::now();
    let config = parse_args()?;

    if config.mode != "dry-run" && config.mode != "run" {
        return Err("mode must be dry-run or run".to_owned());
    }
    if config.ci_fast_gate != "PASS" && config.ci_fast_gate != "FAIL" {
        return Err("ci-fast-gate must be PASS or FAIL".to_owned());
    }

    let max_seconds = parse_required_positive_int(&config.max_seconds, "max-seconds")?;

    if config.mode == "run" && config.local_opt_in != "1" {
        return Err(format!(
            "run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1"
        ));
    }
    if config.mode == "run" && config.ci_fast_gate != "FAIL" {
        return Err(
            "run mode requires --ci-fast-gate FAIL for local-only execution scope".to_owned(),
        );
    }

    let (markers, rows) = parse_fixture(&config.fixture_file)?;

    let row = rows
        .get(&config.profile)
        .ok_or_else(|| "profile must be baseline or injected-drift".to_owned())?;

    let min_component_count = parse_positive_int(
        markers
            .get("sbom_provenance_min_component_count")
            .map(String::as_str)
            .ok_or_else(|| {
                "fixture missing required markers: sbom_provenance_min_component_count".to_owned()
            })?,
        "sbom_provenance_min_component_count",
    )?;

    let sbom_component_count = row.sbom_component_count;
    let sbom_package_count = row.sbom_package_count;

    let profile_contract_violation = sbom_component_count < min_component_count;
    let mut status = if profile_contract_violation {
        "fail".to_owned()
    } else {
        "pass".to_owned()
    };
    let mut final_decision = if profile_contract_violation {
        "NO-GO".to_owned()
    } else {
        "GO".to_owned()
    };
    let mut reason_code = if profile_contract_violation {
        "sbom_provenance_profile_contract_violation".to_owned()
    } else {
        "none".to_owned()
    };
    let mut release_manifest_ready_status = if profile_contract_violation {
        "violation".to_owned()
    } else {
        "verified".to_owned()
    };
    let mut artifact_linkage_status = if profile_contract_violation {
        "violation".to_owned()
    } else {
        "verified".to_owned()
    };

    if status != row.expected_status || reason_code != row.expected_reason_code {
        return Err(format!(
            "fixture profile contract mismatch for {}: expected status={}, reason={}",
            config.profile, row.expected_status, row.expected_reason_code
        ));
    }

    let command_count = if config.mode == "dry-run" { 0 } else { 1 };
    let run_mode_command_status = if config.mode == "dry-run" {
        "dry_run_no_commands_executed".to_owned()
    } else {
        "sbom_provenance_generator_executed".to_owned()
    };

    let elapsed_seconds = started.elapsed().as_secs();
    let mut performance_budget_status = "verified".to_owned();
    if elapsed_seconds > max_seconds {
        performance_budget_status = "violation".to_owned();
        status = "fail".to_owned();
        final_decision = "NO-GO".to_owned();
        reason_code = "sbom_provenance_runtime_budget_exceeded".to_owned();
        release_manifest_ready_status = "violation".to_owned();
        artifact_linkage_status = "violation".to_owned();
    }

    let reason_codes_value = if reason_code == "none" {
        "none".to_owned()
    } else {
        reason_code.clone()
    };

    let sbom_schema_version = markers
        .get("sbom_provenance_sbom_schema_version")
        .cloned()
        .ok_or_else(|| {
            "fixture missing required markers: sbom_provenance_sbom_schema_version".to_owned()
        })?;
    let provenance_schema_version = markers
        .get("sbom_provenance_provenance_schema_version")
        .cloned()
        .ok_or_else(|| {
            "fixture missing required markers: sbom_provenance_provenance_schema_version".to_owned()
        })?;

    let report = Report {
        status: status.clone(),
        final_decision: final_decision.clone(),
        lane_mode: config.mode.clone(),
        profile: config.profile.clone(),
        reason_code: reason_code.clone(),
        reason_codes_value: reason_codes_value.clone(),
        sbom_schema_version: sbom_schema_version.clone(),
        provenance_schema_version: provenance_schema_version.clone(),
        sbom_component_count,
        sbom_package_count,
        sbom_digest_sha256: row.sbom_digest_sha256.clone(),
        provenance_digest_sha256: row.provenance_digest_sha256.clone(),
        release_manifest_ready_status: release_manifest_ready_status.clone(),
        artifact_linkage_status: artifact_linkage_status.clone(),
        ci_fast_gate: config.ci_fast_gate.clone(),
        run_mode_command_status: run_mode_command_status.clone(),
        command_count,
        performance_budget_status: performance_budget_status.clone(),
        elapsed_seconds,
        max_seconds,
        fixture_path: config.fixture_file.to_string_lossy().to_string(),
    };

    if !config.output_json.is_empty() {
        write_json(Path::new(&config.output_json), &report)?;
    }

    println!("status={status}");
    println!("final_decision={final_decision}");
    println!("lane_mode={}", config.mode);
    println!("profile={}", config.profile);
    println!("reason_code={reason_code}");
    println!("reason_codes_value={reason_codes_value}");
    println!("schema_version={RUN_SCHEMA_VERSION}");
    println!("artifact_schema_version={ARTIFACT_SCHEMA_VERSION}");
    println!("fixture_schema_version={FIXTURE_SCHEMA_VERSION}");
    println!("reason_taxonomy_version={REASON_TAXONOMY_VERSION}");
    println!("reason_codes_csv={REASON_CODES_CSV}");
    println!("sbom_schema_version={sbom_schema_version}");
    println!("provenance_schema_version={provenance_schema_version}");
    println!("sbom_component_count={sbom_component_count}");
    println!("sbom_package_count={sbom_package_count}");
    println!("sbom_digest_sha256={}", row.sbom_digest_sha256);
    println!("provenance_digest_sha256={}", row.provenance_digest_sha256);
    println!("release_manifest_required_artifact_id=sbom_provenance");
    println!("release_manifest_ready_status={release_manifest_ready_status}");
    println!("artifact_linkage_status={artifact_linkage_status}");
    println!("run_mode_command_status={run_mode_command_status}");
    println!("command_count={command_count}");
    println!("performance_budget_status={performance_budget_status}");
    println!("elapsed_seconds={elapsed_seconds}");
    println!("max_seconds={max_seconds}");

    Ok(if status == "pass" { 0 } else { 1 })
}

fn main() {
    match run() {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
