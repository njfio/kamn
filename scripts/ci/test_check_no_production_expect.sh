#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_no_production_expect.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_no_production_expect.py"

test_harness_require_executable "$CHECKER" "expected production expect checker wrapper to be executable"

test_harness_require_executable "$PY_CHECKER" "expected production expect checker module to be executable"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

EXPECTED_REASON_TAXONOMY_VERSION="kamn.ci.production-panic-replacement-reason-taxonomy.v1"
EXPECTED_REASON_CODES_CSV="scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default"
EXPECTED_RUNTIME_EVIDENCE_OUTPUTS_CSV="runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv"

BASELINE_REPORT="$TMP_DIR/no-production-expect-baseline-report.json"
baseline_output="$(bash "$CHECKER" --output-json "$BASELINE_REPORT")"

if ! printf '%s\n' "$baseline_output" | grep -q '^status=ok$'; then
  echo "expected status=ok for baseline production panic checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q "^reason_taxonomy_version=${EXPECTED_REASON_TAXONOMY_VERSION}$"; then
  echo "expected deterministic reason taxonomy version marker for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q "^reason_codes_csv=${EXPECTED_REASON_CODES_CSV}$"; then
  echo "expected deterministic reason taxonomy csv marker for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected reason_codes_value=none for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q '^reason_class=stable$'; then
  echo "expected reason_class=stable for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q '^runtime_panic_replacement_evidence_status=verified$'; then
  echo "expected runtime evidence verified marker for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q '^runtime_panic_replacement_evidence_violation_count=0$'; then
  echo "expected runtime evidence violation_count=0 marker for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q '^runtime_panic_replacement_evidence_files_csv=none$'; then
  echo "expected runtime evidence files_csv=none marker for baseline checker path" >&2
  exit 1
fi

if ! printf '%s\n' "$baseline_output" | grep -q "^runtime_panic_replacement_evidence_outputs_csv=${EXPECTED_RUNTIME_EVIDENCE_OUTPUTS_CSV}$"; then
  echo "expected runtime evidence outputs csv marker for baseline checker path" >&2
  exit 1
fi

cat <<'RS' > "$TMP_DIR/failing.rs"
fn panic_path() {
    let _value = std::env::var("X").expect("should fail in production path");
}
RS

set +e
failure_output="$(python3 "$PY_CHECKER" --root "$TMP_DIR" 2>&1)"
failure_code=$?
set -e

if [ "$failure_code" -eq 0 ]; then
  echo "expected checker to fail when production expect() is present" >&2
  exit 1
fi

if ! printf '%s\n' "$failure_output" | grep -q "status=fail"; then
  echo "expected checker to emit status=fail for production expect() violation" >&2
  exit 1
fi

if ! printf '%s\n' "$failure_output" | grep -q '^reason_codes_value=production_expect_reachable$'; then
  echo "expected deterministic reason_codes_value for production expect() violation" >&2
  exit 1
fi

if ! printf '%s\n' "$failure_output" | grep -q '^reason_class=panic_reachability$'; then
  echo "expected reason_class=panic_reachability for production expect() violation" >&2
  exit 1
fi

if ! printf '%s\n' "$failure_output" | grep -q '^runtime_panic_replacement_evidence_status=violation$'; then
  echo "expected runtime evidence violation marker for production expect() path" >&2
  exit 1
fi

rm -f "$TMP_DIR/failing.rs"

cat <<'RS' > "$TMP_DIR/panic_macro.rs"
fn panic_macro_path() {
    panic!("panic macro should fail in production path");
}
RS

set +e
panic_macro_output="$(python3 "$PY_CHECKER" --root "$TMP_DIR" 2>&1)"
panic_macro_code=$?
set -e

if [ "$panic_macro_code" -eq 0 ]; then
  echo "expected checker to fail when production panic! is present" >&2
  exit 1
fi

if ! printf '%s\n' "$panic_macro_output" | grep -q "status=fail"; then
  echo "expected checker to emit status=fail for production panic! violation" >&2
  exit 1
fi

if ! printf '%s\n' "$panic_macro_output" | grep -q '^reason_codes_value=production_panic_macro_reachable$'; then
  echo "expected deterministic reason_codes_value for production panic! violation" >&2
  exit 1
fi

if ! printf '%s\n' "$panic_macro_output" | grep -q '^reason_class=panic_reachability$'; then
  echo "expected reason_class=panic_reachability for production panic! violation" >&2
  exit 1
fi

rm -f "$TMP_DIR/panic_macro.rs"

cat <<'RS' > "$TMP_DIR/unreachable_macro.rs"
fn unreachable_macro_path() {
    unreachable!("unreachable macro should fail in production path");
}
RS

set +e
unreachable_macro_output="$(python3 "$PY_CHECKER" --root "$TMP_DIR" 2>&1)"
unreachable_macro_code=$?
set -e

if [ "$unreachable_macro_code" -eq 0 ]; then
  echo "expected checker to fail when production unreachable! is present" >&2
  exit 1
fi

if ! printf '%s\n' "$unreachable_macro_output" | grep -q "status=fail"; then
  echo "expected checker to emit status=fail for production unreachable! violation" >&2
  exit 1
fi

if ! printf '%s\n' "$unreachable_macro_output" | grep -q '^reason_codes_value=production_unreachable_macro_reachable$'; then
  echo "expected deterministic reason_codes_value for production unreachable! violation" >&2
  exit 1
fi

if ! printf '%s\n' "$unreachable_macro_output" | grep -q '^reason_class=panic_reachability$'; then
  echo "expected reason_class=panic_reachability for production unreachable! violation" >&2
  exit 1
fi

rm -f "$TMP_DIR/unreachable_macro.rs"

cat <<'RS' > "$TMP_DIR/unsafe_fallback.rs"
fn unsafe_fallback_path() -> String {
    std::env::var("KAMN_SIGNER_SECRET").unwrap_or("dev-fallback-secret".to_string())
}
RS

set +e
unsafe_fallback_output="$(python3 "$PY_CHECKER" --root "$TMP_DIR" 2>&1)"
unsafe_fallback_code=$?
set -e

if [ "$unsafe_fallback_code" -eq 0 ]; then
  echo "expected checker to fail when production unsafe fallback default is present" >&2
  exit 1
fi

if ! printf '%s\n' "$unsafe_fallback_output" | grep -q "status=fail"; then
  echo "expected checker to emit status=fail for production unsafe fallback violation" >&2
  exit 1
fi

if ! printf '%s\n' "$unsafe_fallback_output" | grep -q '^reason_codes_value=production_unsafe_env_fallback_default$'; then
  echo "expected deterministic reason_codes_value for production unsafe fallback violation" >&2
  exit 1
fi

if ! printf '%s\n' "$unsafe_fallback_output" | grep -q '^reason_class=unsafe_fallback$'; then
  echo "expected reason_class=unsafe_fallback for production unsafe fallback violation" >&2
  exit 1
fi

rm -f "$TMP_DIR/unsafe_fallback.rs"

set +e
missing_root_output="$(python3 "$PY_CHECKER" --root "$TMP_DIR/does-not-exist" 2>&1)"
missing_root_code=$?
set -e

if [ "$missing_root_code" -eq 0 ]; then
  echo "expected checker to fail when scan root does not exist" >&2
  exit 1
fi

if ! printf '%s\n' "$missing_root_output" | grep -q '^reason_codes_value=scan_root_not_found$'; then
  echo "expected deterministic reason_codes_value for missing root violation" >&2
  exit 1
fi

if ! printf '%s\n' "$missing_root_output" | grep -q '^reason_class=configuration$'; then
  echo "expected reason_class=configuration for missing root violation" >&2
  exit 1
fi

cat <<'RS' > "$TMP_DIR/cfg_test_only.rs"
fn safe_path() -> Result<(), String> {
    std::env::var("X").map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn allows_expect_inside_test_module() {
        let value = Some(1).expect("test-only expect");
        assert_eq!(value, 1);
    }
}
RS

python3 "$PY_CHECKER" --root "$TMP_DIR" >/dev/null

rm -f "$TMP_DIR/cfg_test_only.rs"

cat <<'RS' > "$TMP_DIR/cfg_test_prefix_production_violation.rs"
#[cfg(test)]
use std::sync::Mutex;

fn production_violation() {
    let _value = std::env::var("X").expect("should be detected even after cfg(test) import");
}
RS

set +e
cfg_test_prefix_output="$(python3 "$PY_CHECKER" --root "$TMP_DIR" 2>&1)"
cfg_test_prefix_code=$?
set -e

if [ "$cfg_test_prefix_code" -eq 0 ]; then
  echo "expected checker to fail when production expect() appears after top-level cfg(test) attribute" >&2
  exit 1
fi

if ! printf '%s\n' "$cfg_test_prefix_output" | grep -q '^reason_codes_value=production_expect_reachable$'; then
  echo "expected deterministic reason_codes_value for cfg(test)-prefixed production expect() violation" >&2
  exit 1
fi

if ! printf '%s\n' "$cfg_test_prefix_output" | grep -q '^reason_class=panic_reachability$'; then
  echo "expected reason_class=panic_reachability for cfg(test)-prefixed production expect() violation" >&2
  exit 1
fi

rm -f "$TMP_DIR/cfg_test_prefix_production_violation.rs"

echo "production expect checker tests passed."
