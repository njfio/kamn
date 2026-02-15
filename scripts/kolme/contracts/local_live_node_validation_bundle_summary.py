#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
elapsed_seconds = int(sys.argv[5])
max_seconds = int(sys.argv[6])
budget_status = sys.argv[7]
checkout_path = sys.argv[8]
expected_remote_url = sys.argv[9]
expected_ref = sys.argv[10]
base_url = sys.argv[11]
fork_chain_version = sys.argv[12]
integration_command = sys.argv[13]
integration_policy_command = sys.argv[14]
process_command = sys.argv[15]
process_policy_command = sys.argv[16]
integration_report = sys.argv[17]
integration_policy_report = sys.argv[18]
integration_runtime_policy_report = sys.argv[19]
integration_runtime_live_summary = sys.argv[20]
process_report = sys.argv[21]
process_policy_report = sys.argv[22]
rollback_evidence_file = sys.argv[23]
recovery_evidence_file = sys.argv[24]
checks_path = pathlib.Path(sys.argv[25])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason_code = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason_code,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-live-node-validation-bundle-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "ci_fast_gate_eligible": False,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "integration_command": integration_command,
    "integration_policy_command": integration_policy_command,
    "process_lifecycle_command": process_command,
    "process_lifecycle_policy_command": process_policy_command,
    "integration_report": integration_report,
    "integration_policy_report": integration_policy_report,
    "integration_runtime_policy_report": integration_runtime_policy_report,
    "integration_runtime_commit_live_summary": integration_runtime_live_summary,
    "process_lifecycle_report": process_report,
    "process_lifecycle_policy_report": process_policy_report,
    "rollback_evidence_file": rollback_evidence_file,
    "recovery_evidence_file": recovery_evidence_file,
    "checks": checks,
    "artifact_paths": [
        integration_report,
        integration_policy_report,
        integration_runtime_policy_report,
        integration_runtime_live_summary,
        process_report,
        process_policy_report,
        rollback_evidence_file,
        recovery_evidence_file,
    ],
    "contracts": {
        "ci_fast_gate_scope": "local-only",
        "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
        "bundle_contract": "live_node_release_bundle_v1",
        "live_run_rehearsal_lineage_required": True,
        "rollback_recovery_artifact_lineage_required": True,
        "process_lifecycle_rollback_evidence_option": "--rollback-evidence-file",
        "process_lifecycle_recovery_evidence_option": "--recovery-evidence-file",
    },
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
