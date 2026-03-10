pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub type CommandOutput = Output;

pub const FAST_WORKFLOW: &str = ".github/workflows/ci-fast-gate.yml";
pub const DEEP_WORKFLOW: &str = ".github/workflows/ci-deep-validate.yml";
pub const CI_TOOLS_SCRIPT: &str = "scripts/ci/test_ci_tools.sh";
pub const CI_STRATEGY_DOC: &str = "docs/ci/strategy.md";

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(prefix: &str) -> Self {
        let unique_counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "kamn-{}-{}-{}-{}",
            prefix,
            std::process::id(),
            unique_counter,
            unique_time
        ));
        fs::create_dir_all(&dir).expect("failed to create temporary directory");
        Self { path: dir }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

pub fn read_text(relative: &str) -> String {
    fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
}

pub fn run_command(mut command: Command, context: &str) -> Output {
    command.current_dir(repo_root());
    command
        .output()
        .unwrap_or_else(|error| panic!("failed to run command for {context}: {error}"))
}

pub fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed unexpectedly:\n{}",
        output_text(output)
    );
}

pub fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} succeeded unexpectedly:\n{}",
        output_text(output)
    );
}

pub fn extract_fast_mode_block(ci_tools_script: &str) -> String {
    let start_marker = "if [ \"${KAMN_CI_TOOLS_FAST_MODE:-false}\" = \"true\" ]; then";
    let end_marker = "  echo \"Fast-mode CI tool regression tests passed.\"";
    let start = ci_tools_script
        .find(start_marker)
        .expect("missing fast-mode block start marker")
        + start_marker.len();
    let end = ci_tools_script[start..]
        .find(end_marker)
        .map(|index| start + index)
        .expect("missing fast-mode block end marker");
    ci_tools_script[start..end].to_owned()
}

pub fn assert_contains_all(haystack: &str, needles: &[&str], context: &str) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "{context} missing expected marker: {needle}"
        );
    }
}

pub fn extract_json_string_field(raw_json: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\": \"");
    let start = raw_json.find(&marker)? + marker.len();
    let rest = &raw_json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}
