use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique_counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "kamn-{prefix}-{}-{unique_counter}-{unique_time}",
            std::process::id(),
        ));
        fs::create_dir_all(&dir).expect("failed to create temporary directory");
        Self { path: dir }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_command(mut command: Command, context: &str) -> Output {
    command.current_dir(repo_root());
    command
        .output()
        .unwrap_or_else(|error| panic!("failed to run command for {context}: {error}"))
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed unexpectedly:\n{}",
        output_text(output)
    );
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} succeeded unexpectedly:\n{}",
        output_text(output)
    );
}

fn load_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read file {}: {error}", path.display()))
}

fn extract_json_field_value<'a>(json_text: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let key_index = json_text.find(&key)?;
    let after_key = &json_text[key_index + key.len()..];
    let colon_index = after_key.find(':')?;
    Some(after_key[colon_index + 1..].trim_start())
}

fn json_string_field(json_text: &str, field: &str) -> Option<String> {
    let value = extract_json_field_value(json_text, field)?;
    if !value.starts_with('"') {
        return None;
    }
    let tail = &value[1..];
    let end_index = tail.find('"')?;
    Some(tail[..end_index].to_owned())
}

fn json_u64_field(json_text: &str, field: &str) -> Option<u64> {
    let value = extract_json_field_value(json_text, field)?;
    let digits: String = value
        .chars()
        .skip_while(|character| character.is_whitespace())
        .take_while(|character| character.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

fn run_lane(profile: &str, output_file: &Path) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh")
                .arg("--profile")
                .arg(profile)
                .arg("--mode")
                .arg("dry-run")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--max-seconds")
                .arg("240")
                .arg("--output-json")
                .arg(output_file);
            command
        },
        "sqlite crash-restart local-heavy lane",
    )
}

#[test]
fn unit_sqlite_crash_restart_local_heavy_lane_dry_run_emits_deterministic_artifact_markers() {
    let tmp = TempDir::new("sqlite-crash-restart-unit");
    let report_file = tmp.path().join("sqlite-crash-restart-lane-report.json");

    let output = run_lane("combined", &report_file);
    assert_success(&output, "sqlite crash-restart local-heavy dry-run lane");

    let text = output_text(&output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("lane_mode=dry-run"));
    assert!(text.contains("profile=combined"));
    assert!(text.contains("profile_status=verified"));
    assert!(text.contains("reason_code=none"));

    let payload = load_text(&report_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "artifact_schema_version").as_deref(),
        Some("kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1")
    );
    assert_eq!(
        json_string_field(&payload, "reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1")
    );
    assert_eq!(
        json_string_field(&payload, "reason_codes_csv").as_deref(),
        Some("crash_restart_profile_restart_status_mismatch,crash_restart_profile_corruption_status_mismatch,crash_restart_profile_combined_status_mismatch")
    );
}

#[test]
fn functional_sqlite_crash_restart_local_heavy_lane_profiles_project_deterministic_views() {
    let tmp = TempDir::new("sqlite-crash-restart-functional");
    let restart_report = tmp.path().join("sqlite-crash-restart-profile-restart.json");
    let corruption_report = tmp
        .path()
        .join("sqlite-crash-restart-profile-corruption.json");

    let restart_output = run_lane("restart", &restart_report);
    assert_success(&restart_output, "sqlite crash-restart restart profile lane");
    let restart_payload = load_text(&restart_report);
    assert_eq!(
        json_string_field(&restart_payload, "profile").as_deref(),
        Some("restart")
    );
    assert_eq!(
        json_string_field(&restart_payload, "restart_drill_status").as_deref(),
        Some("verified")
    );
    assert_eq!(
        json_string_field(&restart_payload, "corruption_drill_status").as_deref(),
        Some("not_applicable")
    );

    let corruption_output = run_lane("corruption", &corruption_report);
    assert_success(
        &corruption_output,
        "sqlite crash-restart corruption profile lane",
    );
    let corruption_payload = load_text(&corruption_report);
    assert_eq!(
        json_string_field(&corruption_payload, "profile").as_deref(),
        Some("corruption")
    );
    assert_eq!(
        json_string_field(&corruption_payload, "restart_drill_status").as_deref(),
        Some("not_applicable")
    );
    assert_eq!(
        json_string_field(&corruption_payload, "corruption_drill_status").as_deref(),
        Some("verified")
    );
}

#[test]
fn integration_sqlite_crash_restart_local_heavy_lane_combined_profile_projects_source_schema() {
    let tmp = TempDir::new("sqlite-crash-restart-integration");
    let report_file = tmp.path().join("sqlite-crash-restart-combined.json");

    let output = run_lane("combined", &report_file);
    assert_success(&output, "sqlite crash-restart combined profile lane");

    let payload = load_text(&report_file);
    assert_eq!(
        json_string_field(&payload, "source_report_schema_version").as_deref(),
        Some("kamn.runtime.sqlite-crash-recovery-live-contract-lane-report.v1")
    );
    assert_eq!(json_u64_field(&payload, "source_command_count"), Some(0));
    assert_eq!(
        json_string_field(&payload, "restart_drill_status").as_deref(),
        Some("verified")
    );
    assert_eq!(
        json_string_field(&payload, "corruption_drill_status").as_deref(),
        Some("verified")
    );
    assert_eq!(
        json_string_field(&payload, "profile_status").as_deref(),
        Some("verified")
    );
}

#[test]
fn regression_sqlite_crash_restart_local_heavy_lane_rejects_invalid_profile() {
    let tmp = TempDir::new("sqlite-crash-restart-regression");
    let report_file = tmp.path().join("sqlite-crash-restart-invalid-profile.json");

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh")
                .arg("--profile")
                .arg("invalid")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "sqlite crash-restart invalid profile lane",
    );
    assert_failure(&output, "sqlite crash-restart invalid profile lane");
    assert!(
        output_text(&output).contains("profile must be restart, corruption, or combined"),
        "expected deterministic invalid-profile error marker"
    );
}

#[test]
fn performance_sqlite_crash_restart_local_heavy_lane_dry_run_stays_within_budget() {
    let tmp = TempDir::new("sqlite-crash-restart-performance");
    let report_file = tmp.path().join("sqlite-crash-restart-performance.json");

    let started = Instant::now();
    let output = run_lane("combined", &report_file);
    assert_success(&output, "sqlite crash-restart performance lane");
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 20000,
        "sqlite crash-restart local-heavy dry-run exceeded CI budget: {elapsed_millis}ms"
    );
}
