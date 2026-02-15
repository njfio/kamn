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
max_seconds_per_command = int(sys.argv[7])
budget_status = sys.argv[8]
checkout_path = sys.argv[9]
expected_remote_url = sys.argv[10]
expected_ref = sys.argv[11]
metadata_report = sys.argv[12]
command_output_dir = sys.argv[13]
checks_path = pathlib.Path(sys.argv[14])
cargo_profile = sys.argv[15]
commands = sys.argv[16:]

checkpoints = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 4:
        continue
    check_id, command, check_status, output_file = parts
    checkpoints.append(
        {
            "id": check_id,
            "command": command,
            "status": check_status,
            "output_file": output_file,
        }
    )

summary = {
    "schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds_per_command": max_seconds_per_command,
    "command_count": len(commands),
    "cargo_profile": cargo_profile,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "metadata_report": metadata_report,
    "command_output_dir": command_output_dir,
    "commands": commands,
    "checkpoints": checkpoints,
    "artifact_paths": [
        metadata_report,
        command_output_dir,
    ],
    "evidence_bundle_schema_version": "kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1",
    "evidence_bundle": {
        "schema_version": "kamn.kolme.local-fork-rust-test-matrix-evidence-bundle.v1",
        "summary_schema_version": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
        "status": status,
        "reason_code": reason_code,
        "budget_status": budget_status,
        "command_count": len(commands),
        "artifact_paths": [
            metadata_report,
            command_output_dir,
        ],
    },
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
