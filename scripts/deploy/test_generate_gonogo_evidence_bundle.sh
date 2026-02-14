#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/deploy/generate_gonogo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_gonogo_evidence_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

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

if [ ! -x "$GENERATOR" ]; then
  echo "expected go/no-go evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected go/no-go evidence policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/gonogo-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
    --release-candidate "v1.0.0-rc.1" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:abc123" \
    --ci-fast-gate PASS \
    --ci-deep-lane PASS \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 2
)"

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO decision"

python3 - "$go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
required_markers = {
    "ci_fast_gate",
    "ci_deep_lane",
    "rollback_precheck",
    "rollback_trigger_status",
    "approval_quorum",
    "runtime_image_digest",
}
markers = payload.get("evidence_markers")
if not isinstance(markers, list):
    raise SystemExit("expected go/no-go bundle evidence_markers list")
if set(markers) != required_markers:
    raise SystemExit("expected go/no-go bundle evidence_markers to match required checklist markers")
PY

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected policy check to keep GO decision"

no_go_bundle="$TMP_DIR/gonogo-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --release-candidate "v1.0.0-rc.2" \
    --schema-target-version "1.0.0" \
    --runtime-image-digest "sha256:def456" \
    --ci-fast-gate PASS \
    --ci-deep-lane FAIL \
    --rollback-precheck PASS \
    --rollback-trigger-status CLEAR \
    --required-approvals 2 \
    --received-approvals 1
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected generator to derive NO-GO decision"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO bundle policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected policy check to keep NO-GO decision"

tampered_bundle="$TMP_DIR/gonogo-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from policy checker" >&2
  exit 1
fi

# Regression: #623
if ! printf '%s\n' "$tampered_output" | grep -q "policy decision mismatch"; then
  echo "expected regression guard to catch policy decision mismatch" >&2
  exit 1
fi

tampered_missing_evidence_bundle="$TMP_DIR/gonogo-missing-evidence-marker.json"
cp "$go_bundle" "$tampered_missing_evidence_bundle"
python3 - "$tampered_missing_evidence_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["evidence_markers"] = [marker for marker in payload.get("evidence_markers", []) if marker != "rollback_precheck"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_missing_evidence_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_missing_evidence_bundle" 2>&1)"
tampered_missing_evidence_code=$?
set -e

if [ "$tampered_missing_evidence_code" -eq 0 ]; then
  echo "expected missing-evidence-marker bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_missing_evidence_output" | grep -q "missing required evidence markers"; then
  echo "expected explicit missing-required-evidence-markers error from policy checker" >&2
  exit 1
fi

echo "go/no-go evidence bundle tests passed."
