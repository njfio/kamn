#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"

CHECKER="$ROOT_DIR/scripts/ci/check_touched_rust_size_policy.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_touched_rust_size_policy.py"
THRESHOLD_FILE="$ROOT_DIR/fixtures/ci/touched_rust_size_policy_thresholds.json"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/touched_rust_size_policy_baseline.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected touched-rust size policy checker wrapper to be executable"
test_harness_require_executable "$PY_CHECKER" "expected touched-rust size policy python checker to be executable"
test_harness_require_file "$THRESHOLD_FILE" "expected touched-rust size policy threshold fixture to exist"
test_harness_require_file "$BASELINE_FILE" "expected touched-rust size policy baseline fixture to exist"

REPO_DIR="$TMP_DIR/repo"
mkdir -p "$REPO_DIR/crates/demo/src"
(
  cd "$REPO_DIR"
  git init -q
  git config user.name "KAMN Test"
  git config user.email "kamn-test@example.com"
  cat >crates/demo/src/lib.rs <<'EOF'
pub fn tiny_ok() -> u32 {
    1
}
EOF
  git add crates/demo/src/lib.rs
  git commit -q -m "base"
  git branch -M main
)

cat >"$TMP_DIR/thresholds.json" <<'EOF'
{
  "schema_version": "kamn.ci.touched-rust-size-policy-thresholds.v1",
  "max_file_lines": 6,
  "max_function_lines": 3
}
EOF

cat >"$TMP_DIR/baseline.json" <<'EOF'
{
  "schema_version": "kamn.ci.touched-rust-size-policy-baseline.v1",
  "captured_at": "2026-03-09",
  "max_file_lines": 200,
  "max_function_lines": 25,
  "oversized_files": [],
  "oversized_functions": []
}
EOF

cat >"$REPO_DIR/crates/demo/src/lib.rs" <<'EOF'
pub fn still_ok() -> u32 {
    1
}
EOF

bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --threshold-file "$TMP_DIR/thresholds.json" \
  --baseline-file "$TMP_DIR/baseline.json" \
  --output-json "$TMP_DIR/pass.json" >"$TMP_DIR/pass.out"
grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/pass.out"

cat >"$REPO_DIR/crates/demo/src/lib.rs" <<'EOF'
pub fn too_big_file() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    a + b + c + d
}
EOF

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --threshold-file "$TMP_DIR/thresholds.json" \
  --baseline-file "$TMP_DIR/baseline.json" \
  --output-json "$TMP_DIR/file-fail.json" >"$TMP_DIR/file-fail.out" 2>&1; then
  echo "expected checker to fail for a newly oversized touched file" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/file-fail.out"
grep -q '^reason_codes=touched_rust_size_policy_new_oversized_file$' "$TMP_DIR/file-fail.out"

cat >"$TMP_DIR/thresholds-function.json" <<'EOF'
{
  "schema_version": "kamn.ci.touched-rust-size-policy-thresholds.v1",
  "max_file_lines": 20,
  "max_function_lines": 3
}
EOF

cat >"$REPO_DIR/crates/demo/src/lib.rs" <<'EOF'
pub fn too_big_function() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    a + b + c + d
}
EOF

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref main \
  --threshold-file "$TMP_DIR/thresholds-function.json" \
  --baseline-file "$TMP_DIR/baseline.json" \
  --output-json "$TMP_DIR/function-fail.json" >"$TMP_DIR/function-fail.out" 2>&1; then
  echo "expected checker to fail for a newly oversized touched function" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/function-fail.out"
grep -q '^reason_codes=touched_rust_size_policy_new_oversized_function$' "$TMP_DIR/function-fail.out"

(
  cd "$REPO_DIR"
  git checkout -q main
  cat >crates/demo/src/lib.rs <<'EOF'
pub fn legacy_big() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    a + b + c + d
}
EOF
  git add crates/demo/src/lib.rs
  git commit -q -m "legacy oversized baseline"
)

cat >"$REPO_DIR/crates/demo/src/lib.rs" <<'EOF'
pub fn legacy_big() -> u32 {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    a + b + c + d + e
}
EOF

bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref HEAD~1 \
  --threshold-file "$TMP_DIR/thresholds-function.json" \
  --baseline-file "$TMP_DIR/baseline.json" \
  --output-json "$TMP_DIR/legacy-pass.json" >"$TMP_DIR/legacy-pass.out"
grep -q '^status=pass$' "$TMP_DIR/legacy-pass.out"
grep -q '^reason_codes=none$' "$TMP_DIR/legacy-pass.out"

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref does-not-exist \
  --threshold-file "$TMP_DIR/thresholds-function.json" \
  --baseline-file "$TMP_DIR/baseline.json" \
  --output-json "$TMP_DIR/bad-base.json" >"$TMP_DIR/bad-base.out" 2>&1; then
  echo "expected checker to fail when git base cannot be resolved" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/bad-base.out"
grep -q '^reason_codes=touched_rust_size_policy_git_base_unavailable$' "$TMP_DIR/bad-base.out"

cat >"$TMP_DIR/bad-thresholds.json" <<'EOF'
{
  "schema_version": "kamn.ci.touched-rust-size-policy-thresholds.v1",
  "max_file_lines": 0,
  "max_function_lines": 3
}
EOF

if bash "$CHECKER" \
  --repo-root "$REPO_DIR" \
  --base-ref HEAD~1 \
  --threshold-file "$TMP_DIR/bad-thresholds.json" \
  --baseline-file "$TMP_DIR/baseline.json" \
  --output-json "$TMP_DIR/bad-thresholds.out.json" >"$TMP_DIR/bad-thresholds.out" 2>&1; then
  echo "expected checker to fail for invalid threshold metadata" >&2
  exit 1
fi
grep -q '^status=fail$' "$TMP_DIR/bad-thresholds.out"
grep -q '^reason_codes=touched_rust_size_policy_threshold_invalid$' "$TMP_DIR/bad-thresholds.out"

echo "touched-rust size policy checker tests passed."
