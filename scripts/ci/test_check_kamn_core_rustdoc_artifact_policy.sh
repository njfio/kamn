#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
POLICY_SCRIPT="$ROOT_DIR/scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$POLICY_SCRIPT" ]; then
  echo "expected rustdoc artifact policy checker to be executable" >&2
  exit 1
fi

ARTIFACT_FILE="$TMP_DIR/kamn-core-rustdoc.tar.gz"
printf 'rustdoc-artifact-contract\n' >"$TMP_DIR/content.txt"
tar -czf "$ARTIFACT_FILE" -C "$TMP_DIR" content.txt

ARTIFACT_BYTES="$(wc -c <"$ARTIFACT_FILE" | tr -d '[:space:]')"
ARTIFACT_SHA256="$(sha256sum "$ARTIFACT_FILE" | awk '{print $1}')"

REPORT_FILE="$TMP_DIR/report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$REPORT_FILE" <<JSON
{
  "schema_version": "kamn.ci.kamn-core-rustdoc-artifact-report.v1",
  "status": "pass",
  "crate": "kamn-core",
  "command": "RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps",
  "artifact_path": "$ARTIFACT_FILE",
  "artifact_bytes": $ARTIFACT_BYTES,
  "artifact_sha256": "$ARTIFACT_SHA256",
  "runtime_seconds": 12,
  "max_runtime_seconds": 120,
  "docs_contract_test_count": 2,
  "behavioral_test_count": 2,
  "docs_contract_to_behavioral_ratio": 1.0,
  "max_docs_contract_to_behavioral_ratio": 1.0,
  "rustdoc_navigation_ratio_status": "within",
  "reason_key": "kamn.ci.kamn-core-rustdoc-artifact.ok"
}
JSON

bash "$POLICY_SCRIPT" --report-file "$REPORT_FILE" >"$TMP_DIR/pass.out"
grep -q '^kamn_core_rustdoc_artifact_policy=ok$' "$TMP_DIR/pass.out"
grep -q '^rustdoc_navigation_ratio_status=within$' "$TMP_DIR/pass.out"
grep -Eq '^docs_contract_test_count=[0-9]+$' "$TMP_DIR/pass.out"
grep -Eq '^behavioral_test_count=[1-9][0-9]*$' "$TMP_DIR/pass.out"
grep -Eq '^docs_contract_to_behavioral_ratio=[0-9]+(\.[0-9]+)?$' "$TMP_DIR/pass.out"
grep -Eq '^max_docs_contract_to_behavioral_ratio=[0-9]+(\.[0-9]+)?$' "$TMP_DIR/pass.out"

TAMPERED_REPORT="$TMP_DIR/tampered-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TAMPERED_REPORT" <<JSON
{
  "schema_version": "kamn.ci.kamn-core-rustdoc-artifact-report.v1",
  "status": "pass",
  "crate": "kamn-core",
  "command": "RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps",
  "artifact_path": "$ARTIFACT_FILE",
  "artifact_bytes": $ARTIFACT_BYTES,
  "artifact_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "runtime_seconds": 12,
  "max_runtime_seconds": 120,
  "docs_contract_test_count": 2,
  "behavioral_test_count": 2,
  "docs_contract_to_behavioral_ratio": 1.0,
  "max_docs_contract_to_behavioral_ratio": 1.0,
  "rustdoc_navigation_ratio_status": "within",
  "reason_key": "kamn.ci.kamn-core-rustdoc-artifact.ok"
}
JSON

if bash "$POLICY_SCRIPT" --report-file "$TAMPERED_REPORT" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected policy checker to fail on checksum mismatch" >&2
  exit 1
fi
grep -q 'artifact_sha256 does not match file digest' "$TMP_DIR/fail.out"

RATIO_FAIL_REPORT="$TMP_DIR/ratio-fail-report.json"
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$RATIO_FAIL_REPORT" <<JSON
{
  "schema_version": "kamn.ci.kamn-core-rustdoc-artifact-report.v1",
  "status": "pass",
  "crate": "kamn-core",
  "command": "RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps",
  "artifact_path": "$ARTIFACT_FILE",
  "artifact_bytes": $ARTIFACT_BYTES,
  "artifact_sha256": "$ARTIFACT_SHA256",
  "runtime_seconds": 12,
  "max_runtime_seconds": 120,
  "reason_key": "kamn.ci.kamn-core-rustdoc-artifact.ok",
  "docs_contract_test_count": 5,
  "behavioral_test_count": 1,
  "docs_contract_to_behavioral_ratio": 5.0,
  "max_docs_contract_to_behavioral_ratio": 1.0,
  "rustdoc_navigation_ratio_status": "exceeded"
}
JSON

if bash "$POLICY_SCRIPT" --report-file "$RATIO_FAIL_REPORT" >"$TMP_DIR/ratio-fail.out" 2>&1; then
  echo "expected policy checker to fail on docs-vs-behavioral ratio exceedance" >&2
  exit 1
fi
grep -q '^reason_code=docs_behavioral_ratio_threshold_exceeded$' "$TMP_DIR/ratio-fail.out"

echo "kamn-core rustdoc artifact policy checker tests passed."
