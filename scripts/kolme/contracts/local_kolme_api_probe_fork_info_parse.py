#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

body_path = pathlib.Path(sys.argv[1])
out_path = pathlib.Path(sys.argv[2])

try:
    payload = json.loads(body_path.read_text(encoding="utf-8"))
except json.JSONDecodeError as error:
    raise SystemExit(f"invalid json: {error.msg}") from error

if not isinstance(payload, dict):
    raise SystemExit("fork-info payload must be a JSON object")

first_block = payload.get("first_block")
last_block = payload.get("last_block")
if not isinstance(first_block, int) or not isinstance(last_block, int):
    raise SystemExit("fork-info payload must include integer first_block and last_block")

out_path.write_text(
    f"first_block={first_block}\nlast_block={last_block}\n",
    encoding="utf-8",
)
