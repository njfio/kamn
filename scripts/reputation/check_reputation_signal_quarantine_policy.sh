#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/reputation/check_reputation_signal_quarantine_policy.sh \
    --bundle-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

bundle_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-file)
      bundle_file="${2:-}"
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

if [[ -z "$bundle_file" ]]; then
  usage
  fail "--bundle-file is required"
fi

if [[ ! -f "$bundle_file" ]]; then
  fail "bundle file not found: $bundle_file"
fi

output="$(
  python3 - "$bundle_file" <<'PY'
import json
import pathlib
import re
import sys
from typing import Dict, List


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "generated_at",
    "lane",
    "evidence_key",
    "reason_key",
    "signal_context",
    "signal_integrity",
    "risk_controls",
    "policy_checks",
    "reason_codes",
    "ingestion_action",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.reputation.signal-quarantine-evidence.v1":
    fail("unexpected schema_version for reputation signal quarantine evidence bundle")

lane = payload["lane"]
if lane not in {"contract", "deep"}:
    fail("lane must be contract or deep")

expected_evidence_key = f"reputation_signal_quarantine_contract:{lane}:v1"
if payload["evidence_key"] != expected_evidence_key:
    fail(
        "evidence_key mismatch: "
        f"expected {expected_evidence_key}, found {payload['evidence_key']}"
    )

signal_context = payload["signal_context"]
if not isinstance(signal_context, dict):
    fail("signal_context must be an object")
for field in ("signal_id", "subject_did", "signal_kind", "source_channel", "event_age_seconds"):
    if field not in signal_context:
        fail(f"signal_context missing field: {field}")
if not isinstance(signal_context["signal_id"], str) or not signal_context["signal_id"]:
    fail("signal_context.signal_id must be a non-empty string")
if not isinstance(signal_context["subject_did"], str) or not signal_context["subject_did"]:
    fail("signal_context.subject_did must be a non-empty string")
if signal_context["signal_kind"] not in {"ENDORSEMENT", "DISPUTE", "CAPABILITY", "DELIVERY"}:
    fail("signal_context.signal_kind must be ENDORSEMENT, DISPUTE, CAPABILITY, or DELIVERY")
if signal_context["source_channel"] not in {"TELEGRAM", "DISCORD", "API", "SYSTEM"}:
    fail("signal_context.source_channel must be TELEGRAM, DISCORD, API, or SYSTEM")
if not isinstance(signal_context["event_age_seconds"], int):
    fail("signal_context.event_age_seconds must be an integer")

signal_integrity = payload["signal_integrity"]
if not isinstance(signal_integrity, dict):
    fail("signal_integrity must be an object")
for field in ("payload_sha256", "payload_signature_verified", "nonce_unique"):
    if field not in signal_integrity:
        fail(f"signal_integrity missing field: {field}")
if not isinstance(signal_integrity["payload_sha256"], str):
    fail("signal_integrity.payload_sha256 must be a string")
if signal_integrity["payload_signature_verified"] not in {"PASS", "FAIL"}:
    fail("signal_integrity.payload_signature_verified must be PASS or FAIL")
if not isinstance(signal_integrity["nonce_unique"], bool):
    fail("signal_integrity.nonce_unique must be boolean")

risk_controls = payload["risk_controls"]
if not isinstance(risk_controls, dict):
    fail("risk_controls must be an object")
for field in ("rate_within_threshold", "source_attested", "ci_fast_gate"):
    if field not in risk_controls:
        fail(f"risk_controls missing field: {field}")
if not isinstance(risk_controls["rate_within_threshold"], bool):
    fail("risk_controls.rate_within_threshold must be boolean")
if not isinstance(risk_controls["source_attested"], bool):
    fail("risk_controls.source_attested must be boolean")
if risk_controls["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("risk_controls.ci_fast_gate must be PASS or FAIL")

policy_checks = payload["policy_checks"]
if not isinstance(policy_checks, dict):
    fail("policy_checks must be an object")
required_checks = (
    "did_fields_valid",
    "payload_hash_valid",
    "payload_signature_verified",
    "event_fresh",
    "nonce_unique",
    "rate_within_threshold",
    "source_attested",
    "ci_fast_gate_passed",
)
for field in required_checks:
    if field not in policy_checks:
        fail(f"policy_checks missing field: {field}")
    if not isinstance(policy_checks[field], bool):
        fail(f"policy_checks.{field} must be boolean")

did_pattern = re.compile(r"^did:[a-z0-9]+:[A-Za-z0-9._:-]+$")
hash_pattern = re.compile(r"^sha256:[0-9a-f]{64}$")

derived_checks: Dict[str, bool] = {
    "did_fields_valid": bool(did_pattern.match(signal_context["subject_did"])),
    "payload_hash_valid": bool(hash_pattern.match(signal_integrity["payload_sha256"])),
    "payload_signature_verified": signal_integrity["payload_signature_verified"] == "PASS",
    "event_fresh": 0 <= signal_context["event_age_seconds"] <= 300,
    "nonce_unique": signal_integrity["nonce_unique"],
    "rate_within_threshold": risk_controls["rate_within_threshold"],
    "source_attested": risk_controls["source_attested"],
    "ci_fast_gate_passed": risk_controls["ci_fast_gate"] == "PASS",
}

for key, value in derived_checks.items():
    if policy_checks[key] != value:
        fail(f"policy_checks.{key} does not match derived policy")

expected_decision = "GO" if all(derived_checks.values()) else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")
if actual_decision != expected_decision:
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}"
    )

expected_ingestion_action = "ALLOW" if actual_decision == "GO" else "QUARANTINE"
ingestion_action = payload["ingestion_action"]
if ingestion_action not in {"ALLOW", "QUARANTINE"}:
    fail("ingestion_action must be ALLOW or QUARANTINE")
if ingestion_action != expected_ingestion_action:
    fail(
        "ingestion_action mismatch: "
        f"expected {expected_ingestion_action}, found {ingestion_action}"
    )

reason_key = payload["reason_key"]
if not isinstance(reason_key, str) or not reason_key:
    fail("reason_key must be a non-empty string")
expected_reason_key = f"reputation_signal_quarantine_reason_codes:{actual_decision}:v1"
if reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {reason_key}"
    )

failed_checks: List[str] = []
if not derived_checks["did_fields_valid"]:
    failed_checks.append("did_fields_invalid")
if not derived_checks["payload_hash_valid"]:
    failed_checks.append("payload_hash_invalid")
if not derived_checks["payload_signature_verified"]:
    failed_checks.append("payload_signature_unverified")
if not derived_checks["event_fresh"]:
    failed_checks.append("event_stale")
if not derived_checks["nonce_unique"]:
    failed_checks.append("nonce_replay_detected")
if not derived_checks["rate_within_threshold"]:
    failed_checks.append("burst_threshold_exceeded")
if not derived_checks["source_attested"]:
    failed_checks.append("source_unattested")
if not derived_checks["ci_fast_gate_passed"]:
    failed_checks.append("ci_fast_gate_failed")
failed_checks = sorted(failed_checks)

reason_codes = payload["reason_codes"]
if not isinstance(reason_codes, list):
    fail("reason_codes must be an array")
if not all(isinstance(item, str) and item for item in reason_codes):
    fail("reason_codes must contain non-empty strings")
if reason_codes != sorted(reason_codes):
    fail("reason_codes must be sorted and deterministic")
if reason_codes != failed_checks:
    fail(
        "reason_codes mismatch: "
        f"expected reason_codes={failed_checks}, found {reason_codes}"
    )

failed_checks_value = ",".join(failed_checks) if failed_checks else "none"
print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"schema_version={payload['schema_version']}")
print(f"evidence_key={payload['evidence_key']}")
print(f"reason_key={payload['reason_key']}")
print(f"final_decision={actual_decision}")
print(f"ingestion_action={ingestion_action}")
print(f"failed_checks={failed_checks_value}")
PY
)"

printf '%s\n' "$output"
