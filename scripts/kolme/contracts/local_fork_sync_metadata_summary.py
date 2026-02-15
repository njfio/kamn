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
expected_commit = sys.argv[8]
remote_url = sys.argv[9]
head_ref = sys.argv[10]
head_commit = sys.argv[11]
dirty_checkout = sys.argv[12] == "true"
metadata_verified = sys.argv[13] == "true"
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
    "schema_version": "kamn.kolme.local-fork-sync-metadata-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "checkout_path": checkout_path,
    "expected_remote_url": expected_remote_url,
    "expected_ref": expected_ref,
    "expected_commit": expected_commit,
    "metadata": {
        "remote_url": remote_url,
        "head_ref": head_ref,
        "head_commit": head_commit,
        "dirty_checkout": dirty_checkout,
    },
    "metadata_verified": metadata_verified,
    "checks": checks,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
