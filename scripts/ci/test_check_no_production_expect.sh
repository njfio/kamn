#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_no_production_expect.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_no_production_expect.py"

if [ ! -x "$CHECKER" ]; then
  echo "expected production expect checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$PY_CHECKER" ]; then
  echo "expected production expect checker module to be executable" >&2
  exit 1
fi

bash "$CHECKER" >/dev/null

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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

rm -f "$TMP_DIR/unsafe_fallback.rs"

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

echo "production expect checker tests passed."
