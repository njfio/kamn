#!/usr/bin/env python3
"""Governance quorum attestation replay lane runner and report emitter."""

from __future__ import annotations

from datetime import datetime, timezone
import os
from pathlib import Path
import re
import subprocess
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROOT_DIR = SCRIPT_DIR.parent.parent
VALIDATOR_DOC = ROOT_DIR / "docs/foundation/validator-lifecycle-quorum-reconfiguration.md"
THREAT_DOC = ROOT_DIR / "docs/foundation/threat-control-matrix.md"
PAYLOAD_HASH_PATTERN = re.compile(r"^sha256:[0-9a-f]{64}$")


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/governance/run_quorum_attestation_replay_guard_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def _parse_args(argv: list[str]) -> Path:
    output_file = ROOT_DIR / "governance-quorum-attestation-replay-report.json"
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == "--output-file":
            if index + 1 >= len(argv):
                fail("unknown argument: --output-file")
            output_file = Path(argv[index + 1])
            index += 2
            continue
        if arg in {"--help", "-h"}:
            usage()
            raise SystemExit(0)
        fail(f"unknown argument: {arg}")
    return output_file


def _parse_bool_env(name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"{name} must be true or false")


def _require_non_negative_int(name: str, raw_value: str) -> int:
    if re.fullmatch(r"[0-9]+", raw_value) is None:
        fail(f"{name} must be an integer >= 0")
    return int(raw_value)


def _run_command(command: list[str]) -> int:
    completed = subprocess.run(
        command,
        check=False,
        stdout=subprocess.DEVNULL,
    )
    return completed.returncode


def main(argv: list[str]) -> int:
    output_file = _parse_args(argv)

    max_runtime_seconds = _require_non_negative_int(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_MAX_SECONDS",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_MAX_SECONDS", "180"),
    )

    skip_commands = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS", "false"),
    )
    force_lane_failure = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_LANE_FAILURE",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_LANE_FAILURE", "false"),
    )
    force_missing_keys = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_MISSING_KEYS",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_MISSING_KEYS", "false"),
    )
    force_signature_metadata_invalid = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_SIGNATURE_METADATA_INVALID",
        os.getenv(
            "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_SIGNATURE_METADATA_INVALID",
            "false",
        ),
    )
    force_replay_detected = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED", "false"),
    )
    force_approval_shortfall = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_APPROVAL_SHORTFALL",
        os.getenv(
            "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_APPROVAL_SHORTFALL", "false"
        ),
    )
    force_docs_contract_missing = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_DOCS_CONTRACT_MISSING",
        os.getenv(
            "KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_DOCS_CONTRACT_MISSING", "false"
        ),
    )

    proposal_id = os.getenv(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_PROPOSAL_ID",
        "gov-quorum-attestation-001",
    )
    approval_artifact_id = os.getenv(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_APPROVAL_ARTIFACT_ID",
        "approval-artifact-001",
    )
    payload_hash = os.getenv(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_PAYLOAD_HASH",
        "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )
    approver_dids_csv = os.getenv(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_APPROVER_DIDS",
        "kamn:did:agent:validator-1,kamn:did:agent:validator-2",
    )
    signature_algorithm = os.getenv(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_ALGORITHM",
        "ed25519",
    )
    signature_key_id = os.getenv(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_KEY_ID",
        "governance-signing-key-001",
    )
    signature_signed_at_unix = _require_non_negative_int(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_SIGNED_AT_UNIX",
        os.getenv(
            "KAMN_GOVERNANCE_QUORUM_ATTESTATION_SIGNATURE_SIGNED_AT_UNIX", "1716305100"
        ),
    )
    required_signatures = _require_non_negative_int(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_REQUIRED_SIGNATURES",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_REQUIRED_SIGNATURES", "2"),
    )
    received_signatures = _require_non_negative_int(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_RECEIVED_SIGNATURES",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_RECEIVED_SIGNATURES", "2"),
    )
    replay_detected = _parse_bool_env(
        "KAMN_GOVERNANCE_QUORUM_ATTESTATION_REPLAY_DETECTED",
        os.getenv("KAMN_GOVERNANCE_QUORUM_ATTESTATION_REPLAY_DETECTED", "false"),
    )

    if force_approval_shortfall:
        if required_signatures > 0:
            received_signatures = required_signatures - 1
        else:
            received_signatures = 0

    if force_replay_detected:
        replay_detected = True

    if force_signature_metadata_invalid:
        signature_algorithm = "legacy-rsa"
        signature_key_id = ""

    if force_missing_keys:
        approval_artifact_id = ""
        payload_hash = ""

    commands: list[str] = []
    lane_failed = False
    start_epoch = int(time.time())

    if not skip_commands:
        command = [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "governance_workflow",
            "governance_workflow_functional_submit_vote_execute_flow",
        ]
        commands.append(" ".join(command))
        if _run_command(command) != 0:
            lane_failed = True

        command = [
            "cargo",
            "test",
            "-p",
            "kamn-core",
            "--test",
            "governance_workflow",
            "governance_workflow_regression_rejects_replayed_voter_approval_artifact",
        ]
        commands.append(" ".join(command))
        if _run_command(command) != 0:
            lane_failed = True

    if force_lane_failure:
        lane_failed = True

    docs_contract_present = True
    required_doc_markers = (
        "governance_quorum_attestation_replay_policy_contract.py",
        "governance_quorum_attestation_replay_lane_contract.py",
        "run_quorum_attestation_replay_guard_lane.sh",
        "check_quorum_attestation_replay_policy.sh",
        "run_quorum_attestation_replay_contract_lane.sh",
        "kamn.governance.quorum-attestation-replay-report.v1",
        "governance_quorum_attestation_reason_codes:GO:v1",
        "governance_quorum_attestation_reason_codes:NO-GO:v1",
        "quorum attestation evidence drift and replay attempts must fail closed (`Regression: #911`).",
    )
    validator_text = VALIDATOR_DOC.read_text(encoding="utf-8")
    threat_text = THREAT_DOC.read_text(encoding="utf-8")
    for marker in required_doc_markers:
        if marker not in validator_text or marker not in threat_text:
            docs_contract_present = False
            break

    if force_docs_contract_missing:
        docs_contract_present = False

    required_keys_present = True
    if (
        not proposal_id
        or not approval_artifact_id
        or not payload_hash
        or not approver_dids_csv
    ):
        required_keys_present = False
    if PAYLOAD_HASH_PATTERN.fullmatch(payload_hash) is None:
        required_keys_present = False

    approver_dids = approver_dids_csv.split(",")
    if len(approver_dids) == 0:
        required_keys_present = False
    for did in approver_dids:
        if not did or not did.startswith("kamn:did:agent:"):
            required_keys_present = False

    signature_metadata_valid = True
    if signature_algorithm not in {"ed25519", "secp256k1"}:
        signature_metadata_valid = False
    if not signature_key_id:
        signature_metadata_valid = False
    if signature_signed_at_unix <= 0:
        signature_metadata_valid = False

    approval_quorum_met = not (
        required_signatures < 1 or received_signatures < required_signatures
    )
    replay_guard_passed = not replay_detected

    runtime_seconds = int(time.time()) - start_epoch
    runtime_budget_ok = runtime_seconds <= max_runtime_seconds

    decision_reasons: list[str] = []
    if lane_failed:
        decision_reasons.append("governance_quorum_lane_failed")
    if not required_keys_present:
        decision_reasons.append("quorum_attestation_required_keys_missing")
    if not signature_metadata_valid:
        decision_reasons.append("quorum_attestation_signature_metadata_invalid")
    if not approval_quorum_met:
        decision_reasons.append("quorum_attestation_approval_quorum_missing")
    if not replay_guard_passed:
        decision_reasons.append("quorum_attestation_replay_detected")
    if not docs_contract_present:
        decision_reasons.append("quorum_attestation_docs_contract_missing")
    if not runtime_budget_ok:
        decision_reasons.append("runtime_budget_exceeded")

    final_decision = "GO" if not decision_reasons else "NO-GO"
    reason_key = f"governance_quorum_attestation_reason_codes:{final_decision}:v1"

    payload = {
        "schema_version": "kamn.governance.quorum-attestation-replay-report.v1",
        "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "max_runtime_seconds": max_runtime_seconds,
        "runtime_seconds": runtime_seconds,
        "attestation_bundle": {
            "proposal_id": proposal_id,
            "approval_artifact_id": approval_artifact_id,
            "payload_hash": payload_hash,
            "approver_dids": approver_dids,
            "required_signatures": required_signatures,
            "received_signatures": received_signatures,
            "replay_detected": replay_detected,
            "signature_metadata": {
                "algorithm": signature_algorithm,
                "key_id": signature_key_id,
                "signed_at_unix": signature_signed_at_unix,
            },
        },
        "checks": {
            "lane_failed": lane_failed,
            "required_keys_present": required_keys_present,
            "signature_metadata_valid": signature_metadata_valid,
            "approval_quorum_met": approval_quorum_met,
            "replay_guard_passed": replay_guard_passed,
            "docs_contract_present": docs_contract_present,
            "runtime_budget_ok": runtime_budget_ok,
        },
        "commands": commands,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
        "reason_key": reason_key,
    }
    write_json(output_file, payload)

    print("status=ok")
    print(f"output_file={output_file}")
    print(f"final_decision={final_decision}")
    print(f"reason_key={reason_key}")
    print(f"runtime_seconds={runtime_seconds}")
    print(f"max_runtime_seconds={max_runtime_seconds}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
