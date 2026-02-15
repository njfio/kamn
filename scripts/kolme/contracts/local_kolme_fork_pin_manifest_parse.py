#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import re
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
schema_version = payload.get("schema_version")
fork_remote_url = payload.get("fork_remote_url")
expected_remote_url = payload.get("expected_remote_url")
expected_ref = payload.get("expected_ref")
expected_commit = payload.get("expected_commit")

if schema_version != "kamn.kolme.fork-pin-manifest.v1":
    raise SystemExit("fork pin manifest schema_version must be kamn.kolme.fork-pin-manifest.v1")
if not isinstance(fork_remote_url, str) or not fork_remote_url.strip():
    raise SystemExit("fork pin manifest fork_remote_url must be non-empty")
if not isinstance(expected_remote_url, str) or not expected_remote_url.strip():
    raise SystemExit("fork pin manifest expected_remote_url must be non-empty")
if not isinstance(expected_ref, str) or not expected_ref.startswith("refs/heads/"):
    raise SystemExit("fork pin manifest expected_ref must use refs/heads/* format")
if not isinstance(expected_commit, str) or not re.fullmatch(r"[0-9a-fA-F]{40}", expected_commit):
    raise SystemExit("fork pin manifest expected_commit must be a 40-character hex SHA")

print(schema_version)
print(fork_remote_url.strip())
print(expected_remote_url.strip())
print(expected_ref.strip())
print(expected_commit.strip())
