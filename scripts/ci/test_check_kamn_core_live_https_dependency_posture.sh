#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_kamn_core_live_https_dependency_posture.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_kamn_core_live_https_dependency_posture.py"

if [ ! -x "$CHECKER" ]; then
  echo "expected live-https dependency posture checker wrapper to be executable" >&2
  exit 1
fi

if [ ! -x "$PY_CHECKER" ]; then
  echo "expected live-https dependency posture checker module to be executable" >&2
  exit 1
fi

REPORT_FILE="$(mktemp)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR" "$REPORT_FILE"' EXIT

bash "$CHECKER" --output-json "$REPORT_FILE" >/dev/null
python3 - "$REPORT_FILE" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.ci.kamn-core-live-https-dependency-posture-report.v1"
assert report["status"] == "pass"
assert report["reason_codes"] == ["none"]
assert report["violation_count"] == 0
PY

MANIFEST_FIXTURE="$TMP_DIR/Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$MANIFEST_FIXTURE"
sed -i 's/rustls-pemfile = { version = "2.2.0", optional = true }/rustls-pemfile = { version = "2.2.0", optional = false }/' "$MANIFEST_FIXTURE"

set +e
manifest_failure_output="$(bash "$CHECKER" --cargo-manifest "$MANIFEST_FIXTURE" 2>&1)"
manifest_failure_code=$?
set -e

if [ "$manifest_failure_code" -eq 0 ]; then
  echo "expected checker to fail when rustls-pemfile optional posture drifts" >&2
  exit 1
fi

if ! printf '%s\n' "$manifest_failure_output" | grep -q 'dependency `rustls-pemfile` must declare optional = true'; then
  echo "expected optional-flag drift marker from checker" >&2
  exit 1
fi

README_FIXTURE="$TMP_DIR/README.md"
cp "$ROOT_DIR/README.md" "$README_FIXTURE"
sed -i '/adr-kamn-core-live-tls-transport.md/d' "$README_FIXTURE"

set +e
readme_failure_output="$(bash "$CHECKER" --readme "$README_FIXTURE" 2>&1)"
readme_failure_code=$?
set -e

if [ "$readme_failure_code" -eq 0 ]; then
  echo "expected checker to fail when README ADR contract reference is missing" >&2
  exit 1
fi

if ! printf '%s\n' "$readme_failure_output" | grep -q 'README must link live TLS transport ADR'; then
  echo "expected README ADR-link violation marker from checker" >&2
  exit 1
fi

echo "kamn-core live-https dependency posture checker tests passed."
