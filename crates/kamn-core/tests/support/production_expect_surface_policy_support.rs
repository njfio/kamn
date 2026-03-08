use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const BASELINE_SCHEMA_VERSION: &str = "kamn.ci.production-expect-surface-baseline.v1";
pub const THRESHOLD_SCHEMA_VERSION: &str = "kamn.ci.production-expect-surface-thresholds.v1";
pub const REASON_TAXONOMY_VERSION: &str = "kamn.ci.production-expect-surface-reason-taxonomy.v1";
pub const REASON_CODES_CSV: &str = "baseline_file_missing,baseline_file_invalid,baseline_schema_invalid,baseline_value_invalid,threshold_file_missing,threshold_file_invalid,threshold_schema_invalid,threshold_value_invalid,census_command_failed,census_value_invalid,expect_delta_exceeded,expect_threshold_exceeded_unwaived";

const EXPECT_CALL_TOKEN: &[u8] = b".expect(";

#[derive(Debug, Clone)]
pub struct Baseline {
    pub production_rs_file_count: i64,
    pub production_expect_count: i64,
}

#[derive(Debug, Clone)]
pub struct Thresholds {
    pub allowed_expect_delta_max: i64,
}

#[derive(Debug, Clone)]
pub struct CurrentSurface {
    pub production_rs_file_count: i64,
    pub production_expect_count: i64,
}

#[derive(Debug, Clone)]
pub struct Evaluation {
    pub final_decision: &'static str,
    pub reason_codes: Vec<&'static str>,
}

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn fail(reason_code: &str, detail: &str) -> ! {
    panic!(
        "reason_taxonomy_version={} reason_codes_csv={} reason_code={} detail={}",
        REASON_TAXONOMY_VERSION, REASON_CODES_CSV, reason_code, detail
    );
}

pub fn read_file(path: &Path, reason_code: &str) -> String {
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
    required_value(map, key, reason_code)
        .parse::<i64>()
        .unwrap_or_else(|error| {
            fail(
                reason_code,
                &format!("key {} must parse as integer: {}", key, error),
            )
        })
}

pub fn load_baseline(path: &Path) -> Baseline {
    if !path.is_file() {
        fail(
            "baseline_file_missing",
            &format!("baseline fixture is missing: {}", path.display()),
        );
    }
    let raw = read_file(path, "baseline_file_invalid");
    let map = parse_key_value_fixture(&raw, "baseline_file_invalid");
    let schema_version = required_value(&map, "schema_version", "baseline_schema_invalid");
    if schema_version != BASELINE_SCHEMA_VERSION {
        fail(
            "baseline_schema_invalid",
            &format!(
                "unexpected baseline schema version {} in {}",
                schema_version,
                path.display()
            ),
        );
    }
    let production_rs_file_count =
        required_i64(&map, "production_rs_file_count", "baseline_value_invalid");
    let production_expect_count =
        required_i64(&map, "production_expect_count", "baseline_value_invalid");
    if production_rs_file_count < 0 || production_expect_count < 0 {
        fail(
            "baseline_value_invalid",
            "baseline counts must be non-negative",
        );
    }
    Baseline {
        production_rs_file_count,
        production_expect_count,
    }
}

pub fn load_thresholds(path: &Path) -> Thresholds {
    if !path.is_file() {
        fail(
            "threshold_file_missing",
            &format!("threshold fixture is missing: {}", path.display()),
        );
    }
    let raw = read_file(path, "threshold_file_invalid");
    let map = parse_key_value_fixture(&raw, "threshold_file_invalid");
    let schema_version = required_value(&map, "schema_version", "threshold_schema_invalid");
    if schema_version != THRESHOLD_SCHEMA_VERSION {
        fail(
            "threshold_schema_invalid",
            &format!(
                "unexpected threshold schema version {} in {}",
                schema_version,
                path.display()
            ),
        );
    }
    let reason_taxonomy_version =
        required_value(&map, "reason_taxonomy_version", "threshold_schema_invalid");
    if reason_taxonomy_version != REASON_TAXONOMY_VERSION {
        fail(
            "threshold_schema_invalid",
            &format!(
                "unexpected reason taxonomy version {} in {}",
                reason_taxonomy_version,
                path.display()
            ),
        );
    }
    let reason_codes_csv = required_value(&map, "reason_codes_csv", "threshold_schema_invalid");
    if reason_codes_csv != REASON_CODES_CSV {
        fail(
            "threshold_schema_invalid",
            "reason_codes_csv marker mismatch in threshold fixture",
        );
    }

    let allowed_expect_delta_max =
        required_i64(&map, "allowed_expect_delta_max", "threshold_value_invalid");
    if allowed_expect_delta_max < 0 {
        fail(
            "threshold_value_invalid",
            "allowed expect delta must be non-negative",
        );
    }
    Thresholds {
        allowed_expect_delta_max,
    }
}

fn tracked_source_files() -> Vec<PathBuf> {
    let output = Command::new("git")
        .arg("ls-tree")
        .arg("-r")
        .arg("--name-only")
        .arg("HEAD")
        .arg("crates")
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| fail("census_command_failed", &format!("git ls-tree: {}", error)));
    if !output.status.success() {
        fail(
            "census_command_failed",
            &format!("git ls-tree exited with status {}", output.status),
        );
    }
    let stdout = String::from_utf8(output.stdout).unwrap_or_else(|error| {
        fail(
            "census_command_failed",
            &format!("git ls-tree output is not utf8: {}", error),
        )
    });
    let mut files = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.ends_with(".rs") {
            continue;
        }
        if !trimmed.contains("/src/") || trimmed.contains("/tests/") {
            continue;
        }
        if trimmed.ends_with("_tests.rs") || is_test_only_source_path(trimmed) {
            continue;
        }
        files.push(repo_path(trimmed));
    }
    files.sort();
    files
}

pub fn is_test_only_source_path(relative_path: &str) -> bool {
    if relative_path.starts_with("crates/kamn-e2e-harness/") {
        return true;
    }
    if relative_path
        .split('/')
        .any(|component| component == "main_tests")
    {
        return true;
    }
    let Some(file_name) = Path::new(relative_path)
        .file_name()
        .and_then(|value| value.to_str())
    else {
        return false;
    };
    if !file_name.ends_with(".rs") {
        return false;
    }
    let stem = &file_name[..file_name.len().saturating_sub(3)];
    stem == "tests"
        || stem.starts_with("test_")
        || stem.starts_with("runtime_tests")
        || stem.contains("_tests")
}

fn starts_with(bytes: &[u8], index: usize, needle: &[u8]) -> bool {
    bytes
        .get(index..index + needle.len())
        .is_some_and(|candidate| candidate == needle)
}

fn raw_string_start(bytes: &[u8], index: usize) -> Option<(usize, usize)> {
    let prefix_len = match bytes.get(index..index + 2) {
        Some(b"br") | Some(b"rb") => 2,
        _ if bytes.get(index) == Some(&b'r') => 1,
        _ => return None,
    };
    let mut cursor = index + prefix_len;
    let mut hash_count = 0_usize;
    while bytes.get(cursor) == Some(&b'#') {
        hash_count += 1;
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((prefix_len + hash_count + 1, hash_count))
}

fn char_literal_end(bytes: &[u8], index: usize) -> Option<usize> {
    if bytes.get(index) != Some(&b'\'') {
        return None;
    }
    let mut cursor = index + 1;
    let mut escaped = false;
    while cursor < bytes.len() && bytes[cursor] != b'\n' {
        if escaped {
            escaped = false;
        } else if bytes[cursor] == b'\\' {
            escaped = true;
        } else if bytes[cursor] == b'\'' {
            return Some(cursor + 1);
        }
        cursor += 1;
    }
    None
}

fn closes_raw_string(bytes: &[u8], index: usize, hash_count: usize) -> bool {
    if bytes.get(index) != Some(&b'"') {
        return false;
    }
    (0..hash_count).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'))
}

#[derive(Debug, Default, Clone)]
struct CodeScanState {
    block_comment_depth: usize,
    raw_string_hash_count: Option<usize>,
    in_string: bool,
    escaped: bool,
}

fn consume_active_non_code(
    bytes: &[u8],
    index: usize,
    state: &mut CodeScanState,
) -> Option<usize> {
    if let Some(hash_count) = state.raw_string_hash_count {
        if closes_raw_string(bytes, index, hash_count) {
            state.raw_string_hash_count = None;
            return Some(index + hash_count + 2);
        }
        return Some(index + 1);
    }

    if state.block_comment_depth > 0 {
        if starts_with(bytes, index, b"/*") {
            state.block_comment_depth += 1;
            return Some(index + 2);
        }
        if starts_with(bytes, index, b"*/") {
            state.block_comment_depth -= 1;
            return Some(index + 2);
        }
        return Some(index + 1);
    }

    if !state.in_string {
        return None;
    }
    if state.escaped {
        state.escaped = false;
    } else if bytes[index] == b'\\' {
        state.escaped = true;
    } else if bytes[index] == b'"' {
        state.in_string = false;
    }
    Some(index + 1)
}

fn start_non_code(bytes: &[u8], index: usize, state: &mut CodeScanState) -> Option<usize> {
    if starts_with(bytes, index, b"//") {
        let mut cursor = index + 2;
        while cursor < bytes.len() && bytes[cursor] != b'\n' {
            cursor += 1;
        }
        return Some(cursor);
    }
    if starts_with(bytes, index, b"/*") {
        state.block_comment_depth = 1;
        return Some(index + 2);
    }
    if let Some((prefix_len, hash_count)) = raw_string_start(bytes, index) {
        state.raw_string_hash_count = Some(hash_count);
        return Some(index + prefix_len);
    }
    if let Some(end) = char_literal_end(bytes, index) {
        return Some(end);
    }
    if starts_with(bytes, index, b"b\"") {
        state.in_string = true;
        state.escaped = false;
        return Some(index + 2);
    }
    if bytes[index] == b'"' {
        state.in_string = true;
        state.escaped = false;
        return Some(index + 1);
    }
    None
}

fn consume_non_code(bytes: &[u8], index: usize, state: &mut CodeScanState) -> Option<usize> {
    consume_active_non_code(bytes, index, state).or_else(|| start_non_code(bytes, index, state))
}

fn skip_whitespace(bytes: &[u8], mut index: usize, state: &mut CodeScanState) -> usize {
    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, state) {
            index = next;
            continue;
        }
        if !bytes[index].is_ascii_whitespace() {
            break;
        }
        index += 1;
    }
    index
}

fn skip_attribute(bytes: &[u8], mut index: usize, state: &mut CodeScanState) -> usize {
    let mut depth = 0_i64;
    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, state) {
            index = next;
            continue;
        }
        if bytes[index] == b'[' {
            depth += 1;
        } else if bytes[index] == b']' {
            depth -= 1;
            if depth == 0 {
                return index + 1;
            }
        }
        index += 1;
    }
    bytes.len()
}

fn skip_cfg_test_item(bytes: &[u8], mut index: usize) -> usize {
    let mut state = CodeScanState::default();
    index += "#[cfg(test)]".len();
    index = skip_whitespace(bytes, index, &mut state);
    while starts_with(bytes, index, b"#[") {
        index = skip_attribute(bytes, index + 1, &mut state);
        index = skip_whitespace(bytes, index, &mut state);
    }

    let mut body_depth = 0_i64;
    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, &mut state) {
            index = next;
            continue;
        }
        if body_depth == 0 && bytes[index] == b';' {
            return index + 1;
        }
        if bytes[index] == b'{' {
            body_depth += 1;
        } else if bytes[index] == b'}' {
            body_depth -= 1;
            if body_depth == 0 {
                return index + 1;
            }
        }
        index += 1;
    }
    bytes.len()
}

pub fn count_expect_occurrences_excluding_cfg_test(raw: &str) -> i64 {
    let bytes = raw.as_bytes();
    let mut count = 0_i64;
    let mut index = 0_usize;
    let mut state = CodeScanState::default();

    while index < bytes.len() {
        if let Some(next) = consume_non_code(bytes, index, &mut state) {
            index = next;
            continue;
        }
        if starts_with(bytes, index, b"#[cfg(test)]") {
            index = skip_cfg_test_item(bytes, index);
            continue;
        }
        if starts_with(bytes, index, EXPECT_CALL_TOKEN) {
            count += 1;
            index += EXPECT_CALL_TOKEN.len();
            continue;
        }
        index += 1;
    }

    count
}

pub fn current_surface() -> CurrentSurface {
    let files = tracked_source_files();
    let production_rs_file_count = files.len() as i64;
    let mut production_expect_count = 0_i64;
    for file in files {
        let raw = fs::read_to_string(&file).unwrap_or_else(|error| {
            fail(
                "census_value_invalid",
                &format!("failed to read source file {}: {}", file.display(), error),
            )
        });
        production_expect_count += count_expect_occurrences_excluding_cfg_test(&raw);
    }
    if production_rs_file_count < 0 || production_expect_count < 0 {
        fail(
            "census_value_invalid",
            "production source census produced negative counts",
        );
    }
    CurrentSurface {
        production_rs_file_count,
        production_expect_count,
    }
}

pub fn evaluate_policy(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
) -> Evaluation {
    let expect_delta = current.production_expect_count - baseline.production_expect_count;
    let mut fail_reasons = Vec::new();
    if expect_delta > thresholds.allowed_expect_delta_max {
        fail_reasons.push("expect_delta_exceeded");
    }
    if fail_reasons.is_empty() {
        return Evaluation {
            final_decision: "GO",
            reason_codes: vec!["none"],
        };
    }
    fail_reasons.push("expect_threshold_exceeded_unwaived");
    Evaluation {
        final_decision: "NO-GO",
        reason_codes: fail_reasons,
    }
}
