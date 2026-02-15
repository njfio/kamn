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
selected_probe_command = sys.argv[9]
allow_non_default_probe_command = sys.argv[10] == "true"
checks_path = pathlib.Path(sys.argv[11])
default_checkout_path = sys.argv[12]
default_profile = sys.argv[13]
expected_cargo_bin = sys.argv[14]
expected_component = sys.argv[15]

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
    "schema_version": "kamn.kolme.local-fork-profile-preflight-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "checkout_path": checkout_path,
    "selected_probe_command": selected_probe_command,
    "allow_non_default_probe_command": allow_non_default_probe_command,
    "contracts": {
        "default_checkout_path": default_checkout_path,
        "default_profile": default_profile,
        "expected_cargo_bin": expected_cargo_bin,
        "expected_component": expected_component,
    },
    "checks": checks,
    "artifact_paths": [],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
