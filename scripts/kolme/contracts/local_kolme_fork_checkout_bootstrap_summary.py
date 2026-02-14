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
fork_remote_url = sys.argv[9]
expected_remote_url = sys.argv[10]
expected_ref = sys.argv[11]
expected_commit = sys.argv[12]
commit_pin_enforced = sys.argv[13] == "true"
fork_pin_manifest_file = sys.argv[14]
fork_pin_manifest_schema_version = sys.argv[15]
bootstrap_action = sys.argv[16]
sync_metadata_report = sys.argv[17]
git_version = sys.argv[18]
cargo_version = sys.argv[19]
rustc_version = sys.argv[20]
checks_path = pathlib.Path(sys.argv[21])

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
    "schema_version": "kamn.kolme.local-fork-checkout-bootstrap-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "fork_remote_url": fork_remote_url,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "expected_commit": expected_commit,
    "commit_pin_enforced": commit_pin_enforced,
    "fork_pin_manifest_file": fork_pin_manifest_file,
    "fork_pin_manifest_schema_version": fork_pin_manifest_schema_version,
    "bootstrap_action": bootstrap_action,
    "sync_metadata_report": sync_metadata_report,
    "diagnostics": {
        "git_version": git_version,
        "cargo_version": cargo_version,
        "rustc_version": rustc_version,
    },
    "checks": checks,
    "artifact_paths": [
        sync_metadata_report,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
