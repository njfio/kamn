#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_workspace_license_policy.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$CHECKER" ]; then
  echo "expected workspace license policy checker to be executable" >&2
  exit 1
fi

python3 "$CHECKER" --workspace-root "$ROOT_DIR" --expected-license "Apache-2.0" >/dev/null

MISMATCH_MANIFEST="$TMP_DIR/mismatch.Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$MISMATCH_MANIFEST"
python3 - "$MISMATCH_MANIFEST" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r'^license\s*=\s*".*"$', 'license = "MIT"', text, flags=re.MULTILINE)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" --manifest "$MISMATCH_MANIFEST" --expected-license "Apache-2.0" >"$TMP_DIR/mismatch.out" 2>"$TMP_DIR/mismatch.err"; then
  echo "expected workspace license policy checker to fail on mismatched license field" >&2
  cat "$TMP_DIR/mismatch.out" >&2 || true
  cat "$TMP_DIR/mismatch.err" >&2 || true
  exit 1
fi

if ! grep -q "license_mismatch" "$TMP_DIR/mismatch.err"; then
  echo "expected license_mismatch reason in checker stderr output" >&2
  cat "$TMP_DIR/mismatch.err" >&2 || true
  exit 1
fi

MISSING_MANIFEST="$TMP_DIR/missing.Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$MISSING_MANIFEST"
python3 - "$MISSING_MANIFEST" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r'^license\s*=\s*".*"$\n?', '', text, flags=re.MULTILINE)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" --manifest "$MISSING_MANIFEST" --expected-license "Apache-2.0" >"$TMP_DIR/missing.out" 2>"$TMP_DIR/missing.err"; then
  echo "expected workspace license policy checker to fail on missing license field" >&2
  cat "$TMP_DIR/missing.out" >&2 || true
  cat "$TMP_DIR/missing.err" >&2 || true
  exit 1
fi

if ! grep -q "license_missing" "$TMP_DIR/missing.err"; then
  echo "expected license_missing reason in checker stderr output" >&2
  cat "$TMP_DIR/missing.err" >&2 || true
  exit 1
fi

MALFORMED_MANIFEST="$TMP_DIR/malformed.Cargo.toml"
cat >"$MALFORMED_MANIFEST" <<'EOF'
[package
name = "broken"
EOF

if python3 "$CHECKER" --manifest "$MALFORMED_MANIFEST" --expected-license "Apache-2.0" >"$TMP_DIR/malformed.out" 2>"$TMP_DIR/malformed.err"; then
  echo "expected workspace license policy checker to fail on malformed Cargo manifest" >&2
  cat "$TMP_DIR/malformed.out" >&2 || true
  cat "$TMP_DIR/malformed.err" >&2 || true
  exit 1
fi

if ! grep -q "manifest_invalid_toml" "$TMP_DIR/malformed.err"; then
  echo "expected manifest_invalid_toml reason in checker stderr output" >&2
  cat "$TMP_DIR/malformed.err" >&2 || true
  exit 1
fi

PACKAGE_MISSING_MANIFEST="$TMP_DIR/package-missing.Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$PACKAGE_MISSING_MANIFEST"
python3 - "$PACKAGE_MISSING_MANIFEST" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r'^\[package\]\n', '', text, count=1, flags=re.MULTILINE)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" --manifest "$PACKAGE_MISSING_MANIFEST" --expected-license "Apache-2.0" >"$TMP_DIR/package-missing.out" 2>"$TMP_DIR/package-missing.err"; then
  echo "expected workspace license policy checker to fail when package section is missing" >&2
  cat "$TMP_DIR/package-missing.out" >&2 || true
  cat "$TMP_DIR/package-missing.err" >&2 || true
  exit 1
fi

if ! grep -q "package_section_missing" "$TMP_DIR/package-missing.err"; then
  echo "expected package_section_missing reason in checker stderr output" >&2
  cat "$TMP_DIR/package-missing.err" >&2 || true
  exit 1
fi

MISSING_PATH_MANIFEST="$TMP_DIR/does-not-exist.Cargo.toml"
if python3 "$CHECKER" --manifest "$MISSING_PATH_MANIFEST" --expected-license "Apache-2.0" >"$TMP_DIR/manifest-not-found.out" 2>"$TMP_DIR/manifest-not-found.err"; then
  echo "expected workspace license policy checker to fail on missing manifest path" >&2
  cat "$TMP_DIR/manifest-not-found.out" >&2 || true
  cat "$TMP_DIR/manifest-not-found.err" >&2 || true
  exit 1
fi

if ! grep -q "manifest_not_found" "$TMP_DIR/manifest-not-found.err"; then
  echo "expected manifest_not_found reason in checker stderr output" >&2
  cat "$TMP_DIR/manifest-not-found.err" >&2 || true
  exit 1
fi

echo "workspace license policy checker tests passed."
