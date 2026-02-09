#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/generate_reputation_signal_quarantine_evidence_bundle.sh \
    --output-file <path> \
    --lane contract|deep \
    --signal-id <value> \
    --subject-did <value> \
    --signal-kind ENDORSEMENT|DISPUTE|CAPABILITY|DELIVERY \
    --source-channel TELEGRAM|DISCORD|API|SYSTEM \
    --event-age-seconds <non-negative-int> \
    --payload-sha256 sha256:<64-hex> \
    --payload-signature-verified PASS|FAIL \
    --nonce-unique true|false \
    --rate-within-threshold true|false \
    --source-attested true|false \
    --ci-fast-gate PASS|FAIL
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

normalize_bool() {
  local input="$1"
  case "$input" in
    true|false)
      printf '%s\n' "$input"
      ;;
    *)
      fail "boolean fields must be true or false"
      ;;
  esac
}

require_int() {
  local name="$1"
  local value="$2"
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    fail "${name} must be a non-negative integer"
  fi
}

output_file=""
lane=""
signal_id=""
subject_did=""
signal_kind=""
source_channel=""
event_age_seconds=""
payload_sha256=""
payload_signature_verified=""
nonce_unique=""
rate_within_threshold=""
source_attested=""
ci_fast_gate=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-file)
      output_file="${2:-}"
      shift 2
      ;;
    --lane)
      lane="${2:-}"
      shift 2
      ;;
    --signal-id)
      signal_id="${2:-}"
      shift 2
      ;;
    --subject-did)
      subject_did="${2:-}"
      shift 2
      ;;
    --signal-kind)
      signal_kind="${2:-}"
      shift 2
      ;;
    --source-channel)
      source_channel="${2:-}"
      shift 2
      ;;
    --event-age-seconds)
      event_age_seconds="${2:-}"
      shift 2
      ;;
    --payload-sha256)
      payload_sha256="${2:-}"
      shift 2
      ;;
    --payload-signature-verified)
      payload_signature_verified="${2:-}"
      shift 2
      ;;
    --nonce-unique)
      nonce_unique="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --rate-within-threshold)
      rate_within_threshold="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --source-attested)
      source_attested="$(normalize_bool "${2:-}")"
      shift 2
      ;;
    --ci-fast-gate)
      ci_fast_gate="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$output_file" || -z "$lane" || -z "$signal_id" || -z "$subject_did" || -z "$signal_kind" || -z "$source_channel" || -z "$event_age_seconds" || -z "$payload_sha256" || -z "$payload_signature_verified" || -z "$nonce_unique" || -z "$rate_within_threshold" || -z "$source_attested" || -z "$ci_fast_gate" ]]; then
  usage
  fail "all signal quarantine evidence bundle arguments are required"
fi

case "$lane" in
  contract|deep) ;;
  *)
    fail "lane must be contract or deep"
    ;;
esac

case "$signal_kind" in
  ENDORSEMENT|DISPUTE|CAPABILITY|DELIVERY) ;;
  *)
    fail "signal-kind must be ENDORSEMENT, DISPUTE, CAPABILITY, or DELIVERY"
    ;;
esac

case "$source_channel" in
  TELEGRAM|DISCORD|API|SYSTEM) ;;
  *)
    fail "source-channel must be TELEGRAM, DISCORD, API, or SYSTEM"
    ;;
esac

case "$payload_signature_verified" in
  PASS|FAIL) ;;
  *)
    fail "payload-signature-verified must be PASS or FAIL"
    ;;
esac

case "$ci_fast_gate" in
  PASS|FAIL) ;;
  *)
    fail "ci-fast-gate must be PASS or FAIL"
    ;;
esac

require_int "event-age-seconds" "$event_age_seconds"

mkdir -p "$(dirname "$output_file")"
generated_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

final_decision="$(
  python3 - "$output_file" "$generated_at" "$lane" "$signal_id" "$subject_did" "$signal_kind" "$source_channel" "$event_age_seconds" "$payload_sha256" "$payload_signature_verified" "$nonce_unique" "$rate_within_threshold" "$source_attested" "$ci_fast_gate" <<'PY'
import json
import pathlib
import re
import sys
from typing import Dict, List


def fail(message: str) -> None:
    raise ValueError(message)


(
    output_file,
    generated_at,
    lane,
    signal_id,
    subject_did,
    signal_kind,
    source_channel,
    event_age_seconds_raw,
    payload_sha256,
    payload_signature_verified,
    nonce_unique_raw,
    rate_within_threshold_raw,
    source_attested_raw,
    ci_fast_gate,
) = sys.argv[1:]

if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")
if signal_kind not in {"ENDORSEMENT", "DISPUTE", "CAPABILITY", "DELIVERY"}:
    fail("signal_kind must be ENDORSEMENT, DISPUTE, CAPABILITY, or DELIVERY")
if source_channel not in {"TELEGRAM", "DISCORD", "API", "SYSTEM"}:
    fail("source_channel must be TELEGRAM, DISCORD, API, or SYSTEM")
if payload_signature_verified not in {"PASS", "FAIL"}:
    fail("payload_signature_verified must be PASS or FAIL")
if ci_fast_gate not in {"PASS", "FAIL"}:
    fail("ci_fast_gate must be PASS or FAIL")

event_age_seconds = int(event_age_seconds_raw)
nonce_unique = nonce_unique_raw == "true"
rate_within_threshold = rate_within_threshold_raw == "true"
source_attested = source_attested_raw == "true"

did_pattern = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

policy_checks: Dict[str, bool] = {
    "did_fields_valid": bool(did_pattern.match(subject_did)),
    "payload_hash_valid": bool(hash_pattern.match(payload_sha256)),
    "payload_signature_verified": payload_signature_verified == "PASS",
    "event_fresh": 0 <= event_age_seconds <= 300,
    "nonce_unique": nonce_unique,
    "rate_within_threshold": rate_within_threshold,
    "source_attested": source_attested,
    "ci_fast_gate_passed": ci_fast_gate == "PASS",
}

is_go = all(policy_checks.values())
final_decision = "GO" if is_go else "NO-GO"
ingestion_action = "ALLOW" if is_go else "QUARANTINE"

reason_codes: List[str] = []
if not policy_checks["did_fields_valid"]:
    reason_codes.append("did_fields_invalid")
if not policy_checks["payload_hash_valid"]:
    reason_codes.append("payload_hash_invalid")
if not policy_checks["payload_signature_verified"]:
    reason_codes.append("payload_signature_unverified")
if not policy_checks["event_fresh"]:
    reason_codes.append("event_stale")
if not policy_checks["nonce_unique"]:
    reason_codes.append("nonce_replay_detected")
if not policy_checks["rate_within_threshold"]:
    reason_codes.append("burst_threshold_exceeded")
if not policy_checks["source_attested"]:
    reason_codes.append("source_unattested")
if not policy_checks["ci_fast_gate_passed"]:
    reason_codes.append("ci_fast_gate_failed")
reason_codes = sorted(reason_codes)

reason_key = f"reputation_signal_quarantine_reason_codes:{final_decision}:v1"
evidence_key = f"reputation_signal_quarantine_contract:{lane}:v1"

payload = {
    "schema_version": "kamn.reputation.signal-quarantine-evidence.v1",
    "generated_at": generated_at,
    "lane": lane,
    "evidence_key": evidence_key,
    "reason_key": reason_key,
    "signal_context": {
        "signal_id": signal_id,
        "subject_did": subject_did,
        "signal_kind": signal_kind,
        "source_channel": source_channel,
        "event_age_seconds": event_age_seconds,
    },
    "signal_integrity": {
        "payload_sha256": payload_sha256,
        "payload_signature_verified": payload_signature_verified,
        "nonce_unique": nonce_unique,
    },
    "risk_controls": {
        "rate_within_threshold": rate_within_threshold,
        "source_attested": source_attested,
        "ci_fast_gate": ci_fast_gate,
    },
    "policy_checks": policy_checks,
    "reason_codes": reason_codes,
    "ingestion_action": ingestion_action,
    "final_decision": final_decision,
}

path = pathlib.Path(output_file)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
print(final_decision)
PY
)"

ingestion_action="QUARANTINE"
if [ "$final_decision" = "GO" ]; then
  ingestion_action="ALLOW"
fi

printf 'status=generated\n'
printf 'bundle_file=%s\n' "$output_file"
printf 'schema_version=kamn.reputation.signal-quarantine-evidence.v1\n'
printf 'evidence_key=reputation_signal_quarantine_contract:%s:v1\n' "$lane"
printf 'reason_key=reputation_signal_quarantine_reason_codes:%s:v1\n' "$final_decision"
printf 'ingestion_action=%s\n' "$ingestion_action"
printf 'final_decision=%s\n' "$final_decision"
