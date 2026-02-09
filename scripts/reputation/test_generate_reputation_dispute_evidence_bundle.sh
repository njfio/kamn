#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_dispute_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_dispute_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bundle_file="$TMP_DIR/reputation-dispute-go.json"
generator_output="$(
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --dispute-id "dispute-go-test-001" \
    --subject-did "did:kamn:agent-go-001" \
    --reviewer-did "did:kamn:reviewer-go-001" \
    --dispute-reason-code "QUALITY" \
    --evidence-uri "s3://kamn-audit/reputation/dispute-go-test-001.json" \
    --evidence-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --evidence-hash-verified "PASS" \
    --original-trust-score 610 \
    --proposed-trust-score 560 \
    --max-adjustment-points 90 \
    --policy-window-open true \
    --approval-recorded true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^status=generated$"; then
  echo "expected generated status for reputation dispute go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=reputation_dispute_reason_codes:GO:v1$"; then
  echo "expected GO reason key for reputation dispute go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected GO final decision for reputation dispute go case" >&2
  exit 1
fi

python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys

bundle_path = pathlib.Path(sys.argv[1])
payload = json.loads(bundle_path.read_text(encoding="utf-8"))

assert payload["schema_version"] == "kamn.reputation.dispute-evidence.v1"
assert payload["reason_key"] == "reputation_dispute_reason_codes:GO:v1"
assert payload["policy_checks"]["evidence_hash_matches"] is True
assert payload["policy_checks"]["score_adjustment_within_limit"] is True
assert payload["final_decision"] == "GO"
PY

checker_output="$(bash "$CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$checker_output" | grep -q "^final_decision=GO$"; then
  echo "expected checker GO final decision for reputation dispute go case" >&2
  exit 1
fi

if ! printf '%s\n' "$checker_output" | grep -q "^failed_checks=none$"; then
  echo "expected no failed checks for reputation dispute go case" >&2
  exit 1
fi

tampered_bundle="$TMP_DIR/reputation-dispute-tampered.json"
tampered_output="$(
  bash "$GENERATOR" \
    --output-file "$tampered_bundle" \
    --dispute-id "dispute-tampered-test-001" \
    --subject-did "did:kamn:agent-tampered-001" \
    --reviewer-did "did:kamn:reviewer-tampered-001" \
    --dispute-reason-code "ABUSE" \
    --evidence-uri "s3://kamn-audit/reputation/dispute-tampered-test-001.json" \
    --evidence-sha256 "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    --evidence-hash-verified "FAIL" \
    --original-trust-score 720 \
    --proposed-trust-score 620 \
    --max-adjustment-points 120 \
    --policy-window-open true \
    --approval-recorded true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$tampered_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected NO-GO decision for tampered evidence case" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "^reason_key=reputation_dispute_reason_codes:NO-GO:v1$"; then
  echo "expected NO-GO reason key for tampered evidence case" >&2
  exit 1
fi

tampered_checker_output="$(bash "$CHECKER" --bundle-file "$tampered_bundle")"
if ! printf '%s\n' "$tampered_checker_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected checker NO-GO decision for tampered evidence case" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_checker_output" | grep -q "evidence_hash_verification_failed"; then
  echo "expected failed checks to include evidence hash verification failure" >&2
  exit 1
fi

echo "reputation dispute evidence bundle generator tests passed."
