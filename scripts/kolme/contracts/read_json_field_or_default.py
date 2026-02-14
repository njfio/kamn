#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
field_name = sys.argv[2]
missing_field_value = sys.argv[3]

if not path.exists():
    print("report_missing")
    raise SystemExit(0)

try:
    payload = json.loads(path.read_text(encoding="utf-8"))
except json.JSONDecodeError:
    print("report_invalid_json")
    raise SystemExit(0)

value = payload.get(field_name)
if isinstance(value, str) and value.strip():
    print(value)
else:
    print(missing_field_value)
