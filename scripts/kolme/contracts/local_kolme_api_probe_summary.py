#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

output_path = pathlib.Path(sys.argv[1]).resolve()
mode = sys.argv[2]
status = sys.argv[3]
reason_code = sys.argv[4]
base_url = sys.argv[5]
healthz_path = sys.argv[6]
fork_info_path = sys.argv[7]
fork_chain_version = sys.argv[8]
expected_healthz = sys.argv[9]
elapsed_seconds = int(sys.argv[10])
max_seconds = int(sys.argv[11])
budget_status = sys.argv[12]
fork_first_block_raw = sys.argv[13]
fork_last_block_raw = sys.argv[14]
checks_path = pathlib.Path(sys.argv[15])

checks = []
for raw_line in checks_path.read_text(encoding="utf-8").splitlines():
    if not raw_line.strip():
        continue
    parts = raw_line.split("\t")
    if len(parts) != 3:
        continue
    check_id, command, check_status = parts
    checks.append({"id": check_id, "command": command, "status": check_status})

fork_info = {
    "first_block": int(fork_first_block_raw) if fork_first_block_raw else None,
    "last_block": int(fork_last_block_raw) if fork_last_block_raw else None,
}

summary = {
    "schema_version": "kamn.kolme.local-api-probe-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "base_url": base_url,
    "healthz_path": healthz_path,
    "fork_info_path": fork_info_path,
    "fork_chain_version": fork_chain_version,
    "expected_healthz": expected_healthz,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "budget_status": budget_status,
    "fork_info": fork_info,
    "checks": checks,
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
