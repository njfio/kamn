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
matrix_max_seconds = int(sys.argv[7])
budget_status = sys.argv[8]
checks_path = pathlib.Path(sys.argv[9])
matrix_report = sys.argv[10]
matrix_policy_report = sys.argv[11]
matrix_cargo_profile = sys.argv[12]

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
    "schema_version": "kamn.kolme.local-fork-self-test-summary.v1",
    "mode": mode,
    "status": status,
    "reason_code": reason_code,
    "local_only_enforced": True,
    "elapsed_seconds": elapsed_seconds,
    "max_seconds": max_seconds,
    "matrix_max_seconds": matrix_max_seconds,
    "matrix_cargo_profile": matrix_cargo_profile,
    "budget_status": budget_status,
    "contracts": {
        "matrix_runner": "run_local_kolme_fork_rust_test_matrix_lane.sh",
        "matrix_checker": "check_local_kolme_fork_rust_test_matrix_policy.py",
        "matrix_schema": "kamn.kolme.local-fork-rust-test-matrix-summary.v1",
    },
    "checks": checks,
    "artifact_paths": [
        matrix_report,
        matrix_policy_report,
    ],
}

output_path.parent.mkdir(parents=True, exist_ok=True)
output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
