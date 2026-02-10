#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: check_kamn_core_rustdoc_artifact_policy.sh --report-file <path>
USAGE
}

REPORT_FILE=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --report-file)
      REPORT_FILE="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$REPORT_FILE" ]; then
  usage >&2
  exit 2
fi

if [ ! -f "$REPORT_FILE" ]; then
  echo "report file not found: $REPORT_FILE" >&2
  exit 2
fi

python3 - "$REPORT_FILE" <<'PY'
from __future__ import annotations

import hashlib
import json
import pathlib
import re
import sys

report_path = pathlib.Path(sys.argv[1])

payload = json.loads(report_path.read_text(encoding="utf-8"))

errors: list[str] = []

schema_version = payload.get("schema_version")
if schema_version != "kamn.ci.kamn-core-rustdoc-artifact-report.v1":
    errors.append("schema_version mismatch")

status = payload.get("status")
if status != "pass":
    errors.append("status must be pass")

crate = payload.get("crate")
if crate != "kamn-core":
    errors.append("crate must be kamn-core")

command = payload.get("command")
expected_command = "RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps"
if command != expected_command:
    errors.append("command must match bounded rustdoc invocation")

reason_key = payload.get("reason_key")
if reason_key != "kamn.ci.kamn-core-rustdoc-artifact.ok":
    errors.append("reason_key must be kamn.ci.kamn-core-rustdoc-artifact.ok")

runtime_seconds = payload.get("runtime_seconds")
max_runtime_seconds = payload.get("max_runtime_seconds")
if not isinstance(runtime_seconds, int) or runtime_seconds < 0:
    errors.append("runtime_seconds must be a non-negative integer")
if not isinstance(max_runtime_seconds, int) or max_runtime_seconds < 0:
    errors.append("max_runtime_seconds must be a non-negative integer")
if (
    isinstance(runtime_seconds, int)
    and isinstance(max_runtime_seconds, int)
    and runtime_seconds > max_runtime_seconds
):
    errors.append("runtime_seconds exceeds max_runtime_seconds")

artifact_path_raw = payload.get("artifact_path")
if not isinstance(artifact_path_raw, str) or artifact_path_raw.strip() == "":
    errors.append("artifact_path must be a non-empty string")
    artifact_path = None
else:
    artifact_path = pathlib.Path(artifact_path_raw)
    if artifact_path.suffixes[-2:] != [".tar", ".gz"] and artifact_path.suffix != ".tgz":
        errors.append("artifact_path must reference a tar.gz artifact")
    if not artifact_path.is_file():
        errors.append("artifact_path file does not exist")

artifact_bytes = payload.get("artifact_bytes")
if not isinstance(artifact_bytes, int) or artifact_bytes <= 0:
    errors.append("artifact_bytes must be a positive integer")

artifact_sha256 = payload.get("artifact_sha256")
if not isinstance(artifact_sha256, str) or not re.fullmatch(r"[0-9a-f]{64}", artifact_sha256):
    errors.append("artifact_sha256 must be a lowercase 64-char hex digest")

if artifact_path is not None and artifact_path.is_file():
    actual_bytes = artifact_path.stat().st_size
    if isinstance(artifact_bytes, int) and artifact_bytes != actual_bytes:
        errors.append("artifact_bytes does not match file size")
    actual_sha = hashlib.sha256(artifact_path.read_bytes()).hexdigest()
    if isinstance(artifact_sha256, str) and artifact_sha256 != actual_sha:
        errors.append("artifact_sha256 does not match file digest")

if errors:
    for error in errors:
        print(f"kamn-core rustdoc artifact policy failed: {error}", file=sys.stderr)
    raise SystemExit(1)

print("kamn_core_rustdoc_artifact_policy=ok")
PY
