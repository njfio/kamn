#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
overall_status = sys.argv[3]
readiness_status = sys.argv[4]
reason_code = sys.argv[5]
checks_path = pathlib.Path(sys.argv[6])
artifacts_path = pathlib.Path(sys.argv[7])

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

artifacts = [
    line.strip()
    for line in artifacts_path.read_text(encoding="utf-8").splitlines()
    if line.strip()
]

summary = {
    "schema_version": "kamn.kolme.local-bootstrap-summary.v1",
    "mode": mode,
    "status": overall_status,
    "ready": readiness_status == "ready",
    "readiness_status": readiness_status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "checks": checks,
    "artifact_paths": artifacts,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
