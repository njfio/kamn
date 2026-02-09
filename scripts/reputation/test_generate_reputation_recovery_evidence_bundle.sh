#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_recovery_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_recovery_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected reputation recovery evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected reputation recovery policy checker to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-recovery-go.json"
generator_output="$(
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --lane contract \
    --recovery-id "recovery-go-001" \
    --subject-did "did:kamn:agent-go-001" \
    --reviewer-did "did:kamn:reviewer-go-001" \
    --pre-penalty-trust-score 700 \
    --post-penalty-trust-score 540 \
    --proposed-recovered-trust-score 660 \
    --max-reversal-points 160 \
    --false-positive-confirmed true \
    --reviewer-quorum-satisfied true \
    --audit-evidence-verified PASS \
    --replay-guard-pass true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^status=generated$"; then
  echo "expected generated status for recovery go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=reputation_recovery_reason_codes:GO:v1$"; then
  echo "expected GO reason key for recovery go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^recovery_action=REVERSE_PENALTY$"; then
  echo "expected REVERSE_PENALTY action for recovery go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected GO final decision for recovery go case" >&2
  exit 1
fi

python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys

bundle_path = pathlib.Path(sys.argv[1])
payload = json.loads(bundle_path.read_text(encoding="utf-8"))

assert payload["schema_version"] == "kamn.reputation.recovery-reversal-evidence.v1"
assert payload["reason_key"] == "reputation_recovery_reason_codes:GO:v1"
assert payload["recovery_action"] == "REVERSE_PENALTY"
assert payload["reason_codes"] == []
assert payload["final_decision"] == "GO"
assert payload["policy_checks"]["did_fields_valid"] is True
assert payload["policy_checks"]["false_positive_confirmed"] is True
assert payload["policy_checks"]["reviewer_quorum_satisfied"] is True
assert payload["policy_checks"]["audit_evidence_verified"] is True
assert payload["policy_checks"]["replay_guard_passed"] is True
assert payload["policy_checks"]["reversal_within_limit"] is True
assert payload["policy_checks"]["restored_score_within_bounds"] is True
assert payload["policy_checks"]["ci_fast_gate_passed"] is True
PY

checker_output="$(bash "$CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$checker_output" | grep -q "^final_decision=GO$"; then
  echo "expected checker GO final decision for recovery go case" >&2
  exit 1
fi

if ! printf '%s\n' "$checker_output" | grep -q "^failed_checks=none$"; then
  echo "expected no failed checks for recovery go case" >&2
  exit 1
fi

no_go_bundle="$TMP_DIR/reputation-recovery-no-go.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --lane contract \
    --recovery-id "recovery-no-go-001" \
    --subject-did "did:kamn:agent-no-go-001" \
    --reviewer-did "did:kamn:reviewer-no-go-001" \
    --pre-penalty-trust-score 730 \
    --post-penalty-trust-score 520 \
    --proposed-recovered-trust-score 710 \
    --max-reversal-points 120 \
    --false-positive-confirmed false \
    --reviewer-quorum-satisfied false \
    --audit-evidence-verified FAIL \
    --replay-guard-pass false \
    --ci-fast-gate FAIL
)"

if ! printf '%s\n' "$no_go_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected NO-GO decision for recovery no-go case" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_output" | grep -q "^reason_key=reputation_recovery_reason_codes:NO-GO:v1$"; then
  echo "expected NO-GO reason key for recovery no-go case" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_output" | grep -q "^recovery_action=HOLD_PENALTY$"; then
  echo "expected HOLD_PENALTY action for recovery no-go case" >&2
  exit 1
fi

no_go_checker_output="$(bash "$CHECKER" --bundle-file "$no_go_bundle")"
if ! printf '%s\n' "$no_go_checker_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected checker NO-GO decision for recovery no-go case" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_checker_output" | grep -q "false_positive_not_confirmed"; then
  echo "expected failed checks to include false-positive confirmation failure" >&2
  exit 1
fi

if ! printf '%s\n' "$no_go_checker_output" | grep -q "replay_guard_nonce_reused"; then
  echo "expected failed checks to include replay-guard failure" >&2
  exit 1
fi

echo "reputation recovery evidence bundle generator tests passed."
