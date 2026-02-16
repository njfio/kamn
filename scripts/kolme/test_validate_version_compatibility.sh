#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATOR="$ROOT_DIR/scripts/kolme/validate_version_compatibility.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/kolme_compatibility/version_compatibility_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$VALIDATOR" ]; then
  echo "expected Kolme version compatibility validator to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "expected Kolme version compatibility fixture file to exist" >&2
  exit 1
fi

go_output="$(
  python3 "$VALIDATOR" \
    --kamn-version "1.1.0" \
    --kolme-release-tag "v0.15.2" \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT"
)"
assert_eq "$(extract_value "$go_output" "status")" "ok" "expected supported version pair to pass"
assert_eq "$(extract_value "$go_output" "final_decision")" "GO" "expected GO for supported version pair"
if ! printf '%s\n' "$go_output" | grep -q '^reason_taxonomy_version=kamn.kolme.version-compatibility-reason-taxonomy.v1$'; then
  echo "expected deterministic version-compatibility reason taxonomy marker for supported version pair" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -q '^upgrade_rehearsal_bypass_guard_status=verified$'; then
  echo "expected deterministic upgrade-rehearsal bypass guard marker for supported version pair" >&2
  exit 1
fi

set +e
no_go_output="$(
  python3 "$VALIDATOR" \
    --kamn-version "1.2.0" \
    --kolme-release-tag "v0.14.9" \
    --ci-fast-gate PASS \
    --output-json "$TMP_REPORT" 2>&1
)"
no_go_code=$?
set -e

if [ "$no_go_code" -eq 0 ]; then
  echo "expected unsupported version pair to fail closed" >&2
  exit 1
fi

assert_eq "$(extract_value "$no_go_output" "status")" "fail" "expected unsupported version pair to report fail status"
assert_eq "$(extract_value "$no_go_output" "final_decision")" "NO-GO" "expected NO-GO for unsupported version pair"

if ! printf '%s\n' "$no_go_output" | grep -q "kolme_minor_too_old_for_kamn_minor"; then
  echo "expected explicit minor-version compatibility reason" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q '^reason_taxonomy_version=kamn.kolme.version-compatibility-reason-taxonomy.v1$'; then
  echo "expected deterministic version-compatibility reason taxonomy marker for unsupported version pair" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.version-compatibility-report.v1":
    raise SystemExit("unexpected version compatibility report schema")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected persisted NO-GO decision in report")
if payload.get("reason_taxonomy_version") != "kamn.kolme.version-compatibility-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker in report")
if payload.get("reason_codes_csv") != "unsupported_kamn_major,unsupported_kolme_major,kolme_minor_out_of_supported_window,kolme_minor_too_old_for_kamn_minor,ci_fast_gate_failed":
    raise SystemExit("expected deterministic reason_codes_csv marker in report")
if payload.get("upgrade_rehearsal_bypass_guard_status") != "verified":
    raise SystemExit("expected deterministic upgrade_rehearsal_bypass_guard_status marker in report")
PY

# Regression: #775
if ! printf '%s\n' "$no_go_output" | grep -q "kolme_minor_too_old_for_kamn_minor"; then
  echo "expected historical incompatible upgrade signature to remain blocked" >&2
  exit 1
fi

echo "Kolme version compatibility validator tests passed."
