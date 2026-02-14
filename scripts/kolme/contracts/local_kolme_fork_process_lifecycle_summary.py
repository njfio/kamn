#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
checkout_path = sys.argv[5]
expected_remote_url = sys.argv[6]
expected_ref = sys.argv[7]
base_url = sys.argv[8]
fork_chain_version = sys.argv[9]
elapsed_seconds = int(sys.argv[10])
max_seconds = int(sys.argv[11])
budget_status = sys.argv[12]
serve_command = sys.argv[13]
process_output_file = sys.argv[14]
integration_report = sys.argv[15]
integration_runtime_commit_live_policy_report = sys.argv[16]
integration_runtime_commit_policy_reason_code = sys.argv[17]
integration_runtime_commit_finality_command = sys.argv[18]
integration_runtime_commit_finality_max_seconds = int(sys.argv[19])
integration_runtime_commit_finality_output_file = sys.argv[20]
rollback_evidence_file = sys.argv[21]
recovery_evidence_file = sys.argv[22]
rollback_evidence_status = sys.argv[23]
rollback_evidence_reason_code = sys.argv[24]
recovery_evidence_status = sys.argv[25]
recovery_evidence_reason_code = sys.argv[26]
start_reason_code = sys.argv[27]
readiness_reason_code = sys.argv[28]
integration_reason_code = sys.argv[29]
teardown_reason_code = sys.argv[30]
checks_path = pathlib.Path(sys.argv[31])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, check_reason = parts
    checks.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "reason_code": check_reason,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-fork-process-lifecycle-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "base_url": base_url,
    "fork_chain_version": fork_chain_version,
    "serve_command": serve_command,
    "integration_runtime_commit_finality_enabled": bool(integration_runtime_commit_finality_command),
    "integration_runtime_commit_live_policy_report": integration_runtime_commit_live_policy_report,
    "integration_runtime_commit_policy_reason_code": integration_runtime_commit_policy_reason_code,
    "integration_runtime_commit_finality_command": (
        integration_runtime_commit_finality_command if integration_runtime_commit_finality_command else ""
    ),
    "integration_runtime_commit_finality_max_seconds": integration_runtime_commit_finality_max_seconds,
    "integration_runtime_commit_finality_output_file": (
        integration_runtime_commit_finality_output_file if integration_runtime_commit_finality_command else ""
    ),
    "rollback_evidence_file": rollback_evidence_file,
    "recovery_evidence_file": recovery_evidence_file,
    "rollback_evidence_status": rollback_evidence_status,
    "rollback_evidence_reason_code": rollback_evidence_reason_code,
    "recovery_evidence_status": recovery_evidence_status,
    "recovery_evidence_reason_code": recovery_evidence_reason_code,
    "start_reason_code": start_reason_code,
    "readiness_reason_code": readiness_reason_code,
    "integration_reason_code": integration_reason_code,
    "teardown_reason_code": teardown_reason_code,
    "contracts": {
        "healthz_path": "/healthz",
        "fork_info_path": "/fork-info",
        "runtime_commit_endpoint": "/broadcast/runtime-commit",
        "runtime_commit_method": "POST",
        "integration_runner": "run_local_kamn_live_runtime_integration_lane.sh",
        "runtime_provider_client_contract": "KolmeRuntimeCommitLiveProvider",
        "integration_runtime_commit_live_policy_report_option": "--runtime-commit-live-policy-report",
        "rollback_evidence_option": "--rollback-evidence-file",
        "recovery_evidence_option": "--recovery-evidence-file",
        "rollback_evidence_marker": "kamn.kolme.local-fork-process-lifecycle.rollback-evidence.v1",
        "recovery_evidence_marker": "kamn.kolme.local-fork-process-lifecycle.recovery-evidence.v1",
    },
    "checks": checks,
    "artifact_paths": [
        process_output_file,
        integration_report,
        integration_runtime_commit_live_policy_report,
        rollback_evidence_file,
        recovery_evidence_file,
    ],
}

if integration_runtime_commit_finality_command:
    summary["artifact_paths"].append(integration_runtime_commit_finality_output_file)

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
