#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_kamn_core_live_https_dependency_posture.sh"
PY_CHECKER="$ROOT_DIR/scripts/ci/check_kamn_core_live_https_dependency_posture.py"
TLS_HARDENING_DOC="$ROOT_DIR/docs/security/tls-hardening.md"
CI_STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
RELEASE_CHECKLIST_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"
KOLME_DEVNET_OPS_DOC="$ROOT_DIR/docs/deploy/kolme_devnet_ops.md"

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

pass_output="$(bash "$CHECKER" --output-json "$REPORT_FILE")"
if ! printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected deterministic reason-codes csv marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected deterministic reason-codes value marker on pass output" >&2
  exit 1
fi
python3 - "$REPORT_FILE" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
assert report["schema_version"] == "kamn.ci.kamn-core-live-https-dependency-posture-report.v1"
assert report["status"] == "pass"
assert report["reason_codes"] == ["none"]
assert report["reason_taxonomy_version"] == "kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1"
assert report["reason_codes_csv"] == "none"
assert report["reason_codes_value"] == "none"
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
if ! printf '%s\n' "$manifest_failure_output" | grep -q '^reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker from checker" >&2
  exit 1
fi
if ! printf '%s\n' "$manifest_failure_output" | grep -q '^reason_codes_csv=rustls_pemfile_dependency_optional_flag_mismatch$'; then
  echo "expected deterministic optional-flag reason code from checker" >&2
  exit 1
fi
if ! printf '%s\n' "$manifest_failure_output" | grep -q '^reason_codes_value=rustls_pemfile_dependency_optional_flag_mismatch$'; then
  echo "expected deterministic reason-codes value marker from checker" >&2
  exit 1
fi

ROOT_DRIFT_MANIFEST_FIXTURE="$TMP_DIR/Cargo-root-drift.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$ROOT_DRIFT_MANIFEST_FIXTURE"
python3 - "$ROOT_DRIFT_MANIFEST_FIXTURE" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
content = path.read_text(encoding="utf-8")
content = content.replace(
    'live-https = ["dep:rustls", "dep:rustls-pemfile", "dep:webpki-roots"]',
    'live-https = ["dep:rustls", "dep:rustls-pemfile"]',
)
content = content.replace('webpki-roots = { version = "1.0.3", optional = true }\n', "")
path.write_text(content, encoding="utf-8")
PY

set +e
root_drift_output="$(bash "$CHECKER" --cargo-manifest "$ROOT_DRIFT_MANIFEST_FIXTURE" 2>&1)"
root_drift_code=$?
set -e

if [ "$root_drift_code" -eq 0 ]; then
  echo "expected checker to fail when webpki root mapping+dependency drift is introduced" >&2
  exit 1
fi

if ! printf '%s\n' "$root_drift_output" | grep -q 'live-https feature must include mapping `dep:webpki-roots`'; then
  echo "expected webpki-root feature mapping drift marker from checker" >&2
  exit 1
fi
if ! printf '%s\n' "$root_drift_output" | grep -q 'dependency `webpki-roots` must be declared under \[dependencies\]'; then
  echo "expected webpki-root dependency drift marker from checker" >&2
  exit 1
fi
if ! printf '%s\n' "$root_drift_output" | grep -q '^reason_codes_csv=webpki_roots_dependency_missing,webpki_roots_feature_mapping_missing$'; then
  echo "expected deterministic webpki-root reason taxonomy csv marker from checker" >&2
  exit 1
fi
if ! printf '%s\n' "$root_drift_output" | grep -q '^reason_codes_value=webpki_roots_dependency_missing,webpki_roots_feature_mapping_missing$'; then
  echo "expected deterministic webpki-root reason taxonomy value marker from checker" >&2
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
if ! printf '%s\n' "$readme_failure_output" | grep -q '^reason_codes_csv=readme_adr_link_missing$'; then
  echo "expected deterministic readme ADR-link reason code from checker" >&2
  exit 1
fi

if [ ! -f "$TLS_HARDENING_DOC" ]; then
  echo "expected tls hardening doc to exist" >&2
  exit 1
fi
if ! grep -q "check_kamn_core_live_https_dependency_posture.sh" "$TLS_HARDENING_DOC"; then
  echo "expected tls hardening doc to reference live-https posture checker command" >&2
  exit 1
fi
if ! grep -q "kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1" "$TLS_HARDENING_DOC"; then
  echo "expected tls hardening doc to reference live-https deterministic reason taxonomy version" >&2
  exit 1
fi
if ! grep -q "webpki_roots_dependency_missing" "$TLS_HARDENING_DOC"; then
  echo "expected tls hardening doc to include webpki-root dependency fail-closed reason marker" >&2
  exit 1
fi
if ! grep -q "webpki_roots_feature_mapping_missing" "$TLS_HARDENING_DOC"; then
  echo "expected tls hardening doc to include webpki-root feature-mapping fail-closed reason marker" >&2
  exit 1
fi
if ! grep -q "docs/security/tls-hardening.md" "$CI_STRATEGY_DOC"; then
  echo "expected ci strategy doc to reference tls hardening policy doc" >&2
  exit 1
fi
if ! grep -q "reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1" "$CI_STRATEGY_DOC"; then
  echo "expected ci strategy doc to include deterministic tls dependency-posture reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "TLS Dependency-Posture Gate (Issues #4480, #4481)" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release go/no-go checklist to include tls dependency-posture gate section" >&2
  exit 1
fi
if ! grep -q "rustls_pemfile_dependency_optional_flag_mismatch" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release go/no-go checklist to include tls dependency-posture fail-closed reason markers" >&2
  exit 1
fi
if ! grep -q "webpki_roots_dependency_missing" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release go/no-go checklist to include webpki-root dependency drift reason marker" >&2
  exit 1
fi
if ! grep -q "webpki_roots_feature_mapping_missing" "$RELEASE_CHECKLIST_DOC"; then
  echo "expected release go/no-go checklist to include webpki-root feature-mapping drift reason marker" >&2
  exit 1
fi
if [ ! -f "$KOLME_DEVNET_OPS_DOC" ]; then
  echo "expected kolme devnet ops compatibility doc to exist" >&2
  exit 1
fi
if ! grep -q "check_kamn_core_live_https_dependency_posture.sh" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected kolme devnet ops compatibility doc to reference live-https posture checker command" >&2
  exit 1
fi
if ! grep -q "reason_taxonomy_version=kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected kolme devnet ops compatibility doc to include live-https deterministic reason taxonomy marker" >&2
  exit 1
fi
if ! grep -q "rustls_pemfile_dependency_optional_flag_mismatch" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected kolme devnet ops compatibility doc to include live-https fail-closed reason marker" >&2
  exit 1
fi
if ! grep -q "webpki_roots_dependency_missing" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected kolme devnet ops compatibility doc to include webpki-root dependency drift reason marker" >&2
  exit 1
fi
if ! grep -q "webpki_roots_feature_mapping_missing" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected kolme devnet ops compatibility doc to include webpki-root feature-mapping drift reason marker" >&2
  exit 1
fi
if ! grep -q "Regression: #4108" "$KOLME_DEVNET_OPS_DOC"; then
  echo "expected kolme devnet ops compatibility doc to include live-https runbook-sync regression marker" >&2
  exit 1
fi

echo "kamn-core live-https dependency posture checker tests passed."
