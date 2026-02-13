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
