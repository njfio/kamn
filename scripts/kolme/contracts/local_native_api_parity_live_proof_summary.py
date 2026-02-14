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
nonce_command = sys.argv[9]
broadcast_command = sys.argv[10]
finality_command = sys.argv[11]
nonce_log = sys.argv[12]
broadcast_log = sys.argv[13]
finality_log = sys.argv[14]
checks_path = pathlib.Path(sys.argv[15])

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
    "schema_version": "kamn.kolme.local-native-api-parity-live-proof-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": local_only_enforced,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "nonce_command": nonce_command,
    "broadcast_command": broadcast_command,
    "finality_command": finality_command,
    "checks": checks,
    "artifact_paths": [
        nonce_log,
        broadcast_log,
        finality_log,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
