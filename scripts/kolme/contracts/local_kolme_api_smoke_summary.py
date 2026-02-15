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
base_url = sys.argv[9]
smoke_command = sys.argv[10]
fork_chain_version = sys.argv[11]
probe_report = sys.argv[12]
smoke_output_file = sys.argv[13]
checks_path = pathlib.Path(sys.argv[14])

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

summary = {
    "schema_version": "kamn.kolme.local-api-smoke-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "base_url": base_url,
    "smoke_command": smoke_command,
    "fork_chain_version": fork_chain_version,
    "probe_report": probe_report,
    "smoke_output_file": smoke_output_file,
    "checks": checks,
    "artifact_paths": [
        probe_report,
        smoke_output_file,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
