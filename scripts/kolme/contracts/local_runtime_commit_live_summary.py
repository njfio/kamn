#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
local_only_enforced = sys.argv[5] == "true"
elapsed_seconds = int(sys.argv[6])
max_seconds = int(sys.argv[7])
budget_status = sys.argv[8]
live_command = sys.argv[9]
live_output_file = sys.argv[10]
finality_command = sys.argv[11]
finality_output_file = sys.argv[12]
base_url = sys.argv[13]
provider_hint = sys.argv[14]
preflight_max_seconds = int(sys.argv[15])
finality_max_seconds = int(sys.argv[16])
skip_preflight = sys.argv[17] == "1"
checks_path = pathlib.Path(sys.argv[18])
provider_client_contract = sys.argv[19]
provider_submit_profile_contract = sys.argv[20]
provider_command_marker = sys.argv[21]
provider_command_marker_present = sys.argv[22] == "true"
provider_signing_profile_marker = sys.argv[23]
provider_signing_profile_marker_present = sys.argv[24] == "true"
submit_evidence_marker = sys.argv[25]
submit_evidence_marker_present = sys.argv[26] == "true"
finality_evidence_marker = sys.argv[27]
finality_evidence_marker_present = sys.argv[28] == "true"
native_payload_pubkey_marker = sys.argv[29]
native_payload_pubkey_marker_present = sys.argv[30] == "true"
native_payload_nonce_marker = sys.argv[31]
native_payload_nonce_marker_present = sys.argv[32] == "true"
native_payload_messages_marker = sys.argv[33]
native_payload_messages_marker_present = sys.argv[34] == "true"
request_payload_evidence_marker = sys.argv[35]
request_payload_evidence_marker_present = sys.argv[36] == "true"
request_payload_evidence_artifact_path = sys.argv[37]
submit_evidence_artifact_path = sys.argv[38]
finality_evidence_artifact_path = sys.argv[39]
request_finality_evidence_contract_version = sys.argv[40]
request_finality_evidence_linked = sys.argv[41] == "true"
finality_retry_contract_version = sys.argv[42]
finality_retry_max_attempts = int(sys.argv[43])
finality_retry_backoff_seconds = int(sys.argv[44])
finality_retry_attempts_used = int(sys.argv[45])
finality_retry_exhausted = sys.argv[46] == "true"
finality_retry_failure_class = sys.argv[47]
provider_contract_enforcement_mode = sys.argv[48]
provider_live_contract_marker = sys.argv[49]
provider_live_contract_marker_present = sys.argv[50] == "true"
provider_in_memory_reference_detected = sys.argv[51] == "true"
provider_signer_adapter_contract = sys.argv[52]
provider_signing_curve_contract = sys.argv[53]
provider_signing_profile_contract_version = sys.argv[54]


def classify_synthetic_command(command: str) -> bool:
    normalized = " ".join(command.strip().split())
    if not normalized or normalized == "<not-configured>":
        return False

    lower = normalized.lower()
    executable_markers = (
        "cargo test",
        "curl ",
        "python3 ",
        "bash scripts/",
        "./scripts/",
    )
    has_executable_marker = any(marker in lower for marker in executable_markers)
    synthetic_prefixes = ("printf ", "echo ", "true", ":", "cat <<", "cat<<")
    has_synthetic_prefix = lower.startswith(synthetic_prefixes)

    if has_synthetic_prefix and not has_executable_marker:
        return True
    if has_executable_marker:
        return False
    if "status=submitted" in lower or "finality=final" in lower:
        return True
    return False


checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 3:
        continue
    check_id, command, check_status = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
        }
    )

live_command_synthetic = classify_synthetic_command(live_command)
finality_enabled = bool(finality_command.strip())
if finality_enabled:
    finality_command_synthetic = classify_synthetic_command(finality_command)
else:
    finality_command_synthetic = False

live_output_text = ""
live_output_path = pathlib.Path(live_output_file)
if live_output_path.is_file():
    live_output_text = live_output_path.read_text(encoding="utf-8", errors="replace")

replay_evidence_marker = "replay_guard=verified"
replay_evidence_marker_present = (
    replay_evidence_marker in live_command
    or replay_evidence_marker in live_output_text
)

summary = {
    "schema_version": "kamn.kolme.local-runtime-commit-live-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "live_command": live_command,
    "live_command_synthetic": live_command_synthetic,
    "live_output_file": live_output_file,
    "finality_command": finality_command,
    "finality_command_synthetic": finality_command_synthetic,
    "finality_output_file": finality_output_file,
    "finality_enabled": finality_enabled,
    "finality_max_seconds": finality_max_seconds,
    "finality_retry_contract_version": finality_retry_contract_version,
    "finality_retry_max_attempts": finality_retry_max_attempts,
    "finality_retry_backoff_seconds": finality_retry_backoff_seconds,
    "finality_retry_attempts_used": finality_retry_attempts_used,
    "finality_retry_exhausted": finality_retry_exhausted,
    "finality_retry_failure_class": finality_retry_failure_class,
    "synthetic_evidence_classification_version": "v1",
    "base_url": base_url,
    "provider_hint": provider_hint,
    "provider_client_contract": provider_client_contract,
    "provider_contract_enforcement_mode": provider_contract_enforcement_mode,
    "provider_live_contract_marker": provider_live_contract_marker,
    "provider_live_contract_marker_present": provider_live_contract_marker_present,
    "provider_in_memory_reference_detected": provider_in_memory_reference_detected,
    "provider_signer_adapter_contract": provider_signer_adapter_contract,
    "provider_signing_curve_contract": provider_signing_curve_contract,
    "provider_signing_profile_contract_version": provider_signing_profile_contract_version,
    "provider_submit_profile_contract": provider_submit_profile_contract,
    "provider_command_marker": provider_command_marker,
    "provider_command_marker_present": provider_command_marker_present,
    "provider_signing_profile_marker": provider_signing_profile_marker,
    "provider_signing_profile_marker_present": provider_signing_profile_marker_present,
    "submit_evidence_marker": submit_evidence_marker,
    "submit_evidence_marker_present": submit_evidence_marker_present,
    "finality_evidence_marker": finality_evidence_marker,
    "finality_evidence_marker_present": finality_evidence_marker_present,
    "replay_evidence_marker": replay_evidence_marker,
    "replay_evidence_marker_present": replay_evidence_marker_present,
    "replay_evidence_contract_version": "v1",
    "native_payload_pubkey_marker": native_payload_pubkey_marker,
    "native_payload_pubkey_marker_present": native_payload_pubkey_marker_present,
    "native_payload_nonce_marker": native_payload_nonce_marker,
    "native_payload_nonce_marker_present": native_payload_nonce_marker_present,
    "native_payload_messages_marker": native_payload_messages_marker,
    "native_payload_messages_marker_present": native_payload_messages_marker_present,
    "native_payload_marker_contract_version": "v1",
    "request_payload_evidence_marker": request_payload_evidence_marker,
    "request_payload_evidence_marker_present": request_payload_evidence_marker_present,
    "request_payload_evidence_artifact_path": request_payload_evidence_artifact_path,
    "submit_evidence_artifact_path": submit_evidence_artifact_path,
    "finality_evidence_artifact_path": finality_evidence_artifact_path,
    "request_finality_evidence_contract_version": request_finality_evidence_contract_version,
    "request_finality_evidence_linked": request_finality_evidence_linked,
    "preflight_enabled": not skip_preflight,
    "preflight_max_seconds": preflight_max_seconds,
    "checks": checks,
    "artifact_paths": [
        live_output_file,
        finality_output_file,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
