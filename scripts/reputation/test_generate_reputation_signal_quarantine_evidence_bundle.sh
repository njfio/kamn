#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/reputation/generate_reputation_signal_quarantine_evidence_bundle.sh"
CHECKER="$ROOT_DIR/scripts/reputation/check_reputation_signal_quarantine_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$GENERATOR" ]; then
  echo "expected reputation signal quarantine evidence generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected reputation signal quarantine policy checker to be executable" >&2
  exit 1
fi

bundle_file="$TMP_DIR/reputation-signal-quarantine-go.json"
generator_output="$(
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --lane contract \
    --signal-id "signal-go-001" \
    --subject-did "did:kamn:agent-go-001" \
    --signal-kind "ENDORSEMENT" \
    --source-channel "DISCORD" \
    --event-age-seconds 45 \
    --payload-sha256 "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
    --payload-signature-verified PASS \
    --nonce-unique true \
    --rate-within-threshold true \
    --source-attested true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^status=generated$"; then
  echo "expected generated status for signal quarantine go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=reputation_signal_quarantine_reason_codes:GO:v1$"; then
  echo "expected GO reason key for signal quarantine go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^ingestion_action=ALLOW$"; then
  echo "expected ALLOW ingestion action for signal quarantine go case" >&2
  exit 1
fi

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected GO final decision for signal quarantine go case" >&2
  exit 1
fi

python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys

bundle_path = pathlib.Path(sys.argv[1])
payload = json.loads(bundle_path.read_text(encoding="utf-8"))

assert payload["schema_version"] == "kamn.reputation.signal-quarantine-evidence.v1"
assert payload["reason_key"] == "reputation_signal_quarantine_reason_codes:GO:v1"
assert payload["ingestion_action"] == "ALLOW"
assert payload["reason_codes"] == []
assert payload["final_decision"] == "GO"
assert payload["policy_checks"]["did_fields_valid"] is True
assert payload["policy_checks"]["payload_hash_valid"] is True
assert payload["policy_checks"]["payload_signature_verified"] is True
assert payload["policy_checks"]["event_fresh"] is True
assert payload["policy_checks"]["nonce_unique"] is True
assert payload["policy_checks"]["rate_within_threshold"] is True
assert payload["policy_checks"]["source_attested"] is True
assert payload["policy_checks"]["ci_fast_gate_passed"] is True
PY

checker_output="$(bash "$CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$checker_output" | grep -q "^final_decision=GO$"; then
  echo "expected checker GO final decision for signal quarantine go case" >&2
  exit 1
fi

if ! printf '%s\n' "$checker_output" | grep -q "^failed_checks=none$"; then
  echo "expected no failed checks for signal quarantine go case" >&2
  exit 1
fi

quarantine_bundle="$TMP_DIR/reputation-signal-quarantine-no-go.json"
quarantine_output="$(
  bash "$GENERATOR" \
    --output-file "$quarantine_bundle" \
    --lane contract \
    --signal-id "signal-no-go-001" \
    --subject-did "agent-no-go-001" \
    --signal-kind "DISPUTE" \
    --source-channel "API" \
    --event-age-seconds 1900 \
    --payload-sha256 "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
    --payload-signature-verified FAIL \
    --nonce-unique false \
    --rate-within-threshold false \
    --source-attested false \
    --ci-fast-gate FAIL
)"

if ! printf '%s\n' "$quarantine_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected NO-GO decision for signal quarantine case" >&2
  exit 1
fi

if ! printf '%s\n' "$quarantine_output" | grep -q "^reason_key=reputation_signal_quarantine_reason_codes:NO-GO:v1$"; then
  echo "expected NO-GO reason key for signal quarantine case" >&2
  exit 1
fi

if ! printf '%s\n' "$quarantine_output" | grep -q "^ingestion_action=QUARANTINE$"; then
  echo "expected QUARANTINE ingestion action for signal quarantine case" >&2
  exit 1
fi

quarantine_checker_output="$(bash "$CHECKER" --bundle-file "$quarantine_bundle")"
if ! printf '%s\n' "$quarantine_checker_output" | grep -q "^final_decision=NO-GO$"; then
  echo "expected checker NO-GO decision for signal quarantine case" >&2
  exit 1
fi

if ! printf '%s\n' "$quarantine_checker_output" | grep -q "nonce_replay_detected"; then
  echo "expected failed checks to include nonce replay detection" >&2
  exit 1
fi

if ! printf '%s\n' "$quarantine_checker_output" | grep -q "event_stale"; then
  echo "expected failed checks to include event freshness failure" >&2
  exit 1
fi

echo "reputation signal quarantine evidence bundle generator tests passed."
