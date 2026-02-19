use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const CI_TOOLS_SCRIPT: &str = "scripts/ci/test_ci_tools.sh";
const CI_STRATEGY_DOC: &str = "docs/ci/strategy.md";
const README_DOC: &str = "README.md";
const MAKEFILE: &str = "Makefile";
const QUARANTINE_SCRIPT: &str = "scripts/ci/run_cargo_test_with_quarantine.sh";

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
            "kamn-{}-{}-{}-{}",
            prefix,
            std::process::id(),
            unique_counter,
            unique_time
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

fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn read_text(relative: &str) -> String {
    fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
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

fn assert_contains_all(haystack: &str, needles: &[&str], context: &str) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "{context} missing expected marker: {needle}"
        );
    }
}

fn assert_make_dry_run_contains(target: &str, expected_snippet: &str) {
    let output = run_command(
        {
            let mut command = Command::new("make");
            command.arg("-n").arg(target);
            command
        },
        &format!("make -n {target}"),
    );
    assert_success(&output, &format!("make -n {target}"));
    let output_text = output_text(&output);
    assert!(
        output_text.contains(expected_snippet),
        "make -n {target} missing expected snippet '{expected_snippet}'. output:\n{output_text}"
    );
}

#[test]
fn spec_c01_wave2_inventory_removed_and_ci_tools_wired_to_rust_suite() {
    let removed_wrappers = [
        "scripts/ci/test_makefile_command_surface_contract.sh",
        "scripts/ci/test_makefile_execution_contract.sh",
        "scripts/ci/test_run_cargo_test_with_quarantine.sh",
    ];
    for wrapper in removed_wrappers {
        assert!(
            !repo_path(wrapper).exists(),
            "wave-2 wrapper should be removed: {wrapper}"
        );
    }

    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    assert!(
        ci_tools.contains("cargo test -p kamn-core --test shell_test_surface_migration_wave2"),
        "ci tools command surface must run wave-2 rust migration suite"
    );
    assert!(
        !ci_tools.contains("test_run_cargo_test_with_quarantine.sh"),
        "ci tools command surface should not reference deleted wave-2 quarantine wrapper"
    );
    assert!(
        !ci_tools.contains("test_makefile_command_surface_contract.sh"),
        "ci tools command surface should not reference deleted makefile command-surface wrapper"
    );
    assert!(
        !ci_tools.contains("test_makefile_execution_contract.sh"),
        "ci tools command surface should not reference deleted makefile execution wrapper"
    );
}

#[test]
fn spec_c02_makefile_command_surface_contract_parity() {
    let makefile = read_text(MAKEFILE);
    let required_targets = [
        "check",
        "test",
        "smoke-live-network",
        "deep-live-network",
        "demo",
        "demo-localhost-transport",
        "ci-tools",
    ];

    for target in required_targets {
        assert!(
            makefile.contains(&format!("{target}:")),
            "Makefile command-surface contract failed: missing target '{target}'."
        );
    }

    let required_help_snippets = [
        "make check",
        "make test",
        "make smoke-live-network",
        "make deep-live-network",
        "make demo",
        "make demo-localhost-transport",
        "make ci-tools",
    ];
    assert_contains_all(
        &makefile,
        &required_help_snippets,
        "makefile help command surface",
    );

    let required_command_snippets = [
        "cargo fmt --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test",
        "bash scripts/runtime/run_live_network_smoke_lane.sh --output-json /tmp/live-network-smoke-report.json",
        "bash scripts/runtime/run_live_network_pilot_deep_lane.sh --event-name workflow_dispatch --output-json /tmp/live-network-pilot-report.json",
        "bash scripts/sdk/run_localhost_signed_demo.sh",
        "bash scripts/ci/test_ci_tools.sh",
    ];
    assert_contains_all(
        &makefile,
        &required_command_snippets,
        "makefile command snippet surface",
    );
}

#[test]
fn spec_c03_makefile_execution_contract_parity() {
    assert_make_dry_run_contains("check", "cargo fmt --check");
    assert_make_dry_run_contains(
        "check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
    );
    assert_make_dry_run_contains("test", "cargo test");
    assert_make_dry_run_contains("demo", "bash scripts/sdk/run_localhost_signed_demo.sh");
    assert_make_dry_run_contains(
        "demo-localhost-transport",
        "bash scripts/sdk/run_localhost_signed_demo.sh",
    );
}

#[test]
fn spec_c04_run_cargo_test_with_quarantine_contract_parity() {
    let tmp = TempDir::new("quarantine-wave2");
    let script = repo_path(QUARANTINE_SCRIPT);

    let registry_path = tmp.path().join("flaky-tests.txt");
    fs::write(
        &registry_path,
        "# owner|test-id|issue|expiry|notes\nqa|crate::tests::flaky_a|#180|2099-12-31|tracked quarantine entry\nqa|crate::tests::flaky_b|#181|2099-12-31|another tracked entry\n",
    )
    .expect("failed to write flaky registry");

    let dry_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&script)
                .arg("--registry")
                .arg(&registry_path)
                .arg("--dry-run")
                .arg("--")
                .arg("cargo")
                .arg("test")
                .arg("-p")
                .arg("kamn-core")
                .arg("--test")
                .arg("invariant_harness");
            command
        },
        "run_cargo_test_with_quarantine dry-run non-empty registry",
    );
    assert_success(
        &dry_output,
        "run_cargo_test_with_quarantine dry-run non-empty registry",
    );
    let dry_output_text = output_text(&dry_output);
    assert_contains_all(
        &dry_output_text,
        &[
            "--skip crate::tests::flaky_a",
            "--skip crate::tests::flaky_b",
        ],
        "run_cargo_test_with_quarantine non-empty registry output",
    );

    let empty_registry_path = tmp.path().join("empty-flaky-tests.txt");
    fs::write(&empty_registry_path, "# owner|test-id|issue|expiry|notes\n")
        .expect("failed to write empty flaky registry");

    let dry_output_empty = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&script)
                .arg("--registry")
                .arg(&empty_registry_path)
                .arg("--dry-run")
                .arg("--")
                .arg("cargo")
                .arg("test")
                .arg("-p")
                .arg("kamn-core")
                .arg("--test")
                .arg("invariant_harness");
            command
        },
        "run_cargo_test_with_quarantine dry-run empty registry",
    );
    assert_success(
        &dry_output_empty,
        "run_cargo_test_with_quarantine dry-run empty registry",
    );
    assert!(
        !output_text(&dry_output_empty).contains("--skip "),
        "did not expect skip flags when registry is empty"
    );

    let invalid_command_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&script)
                .arg("--registry")
                .arg(&registry_path)
                .arg("--dry-run")
                .arg("--")
                .arg("cargo")
                .arg("clippy");
            command
        },
        "run_cargo_test_with_quarantine invalid command guard",
    );
    assert_failure(
        &invalid_command_output,
        "run_cargo_test_with_quarantine invalid command guard",
    );
}

#[test]
fn spec_c05_strategy_and_readme_require_wave2_rust_lane_markers() {
    let strategy = read_text(CI_STRATEGY_DOC);
    let readme = read_text(README_DOC);

    assert!(
        strategy.contains("shell_test_surface_migration_wave2"),
        "ci strategy must reference shell test migration wave-2 rust suite"
    );
    assert!(
        readme.contains("shell_test_surface_migration_wave2"),
        "README must reference shell test migration wave-2 rust suite"
    );

    for removed_snippet in [
        "test_makefile_command_surface_contract.sh",
        "test_makefile_execution_contract.sh",
        "test_run_cargo_test_with_quarantine.sh",
    ] {
        assert!(
            !strategy.contains(removed_snippet),
            "ci strategy should not reference removed wave-2 wrapper: {removed_snippet}"
        );
        assert!(
            !readme.contains(removed_snippet),
            "README should not reference removed wave-2 wrapper: {removed_snippet}"
        );
    }
}
