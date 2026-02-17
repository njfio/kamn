#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"

if [[ ! -x "$DISPATCHER" ]]; then
  echo "expected exec dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [[ ! -f "$REGISTRY" ]]; then
  echo "expected exec registry file: $REGISTRY" >&2
  exit 1
fi

python3 - "$ROOT_DIR" "$DISPATCHER" "$REGISTRY" <<'PY'
import json
import shlex
import sys
from pathlib import Path

root = Path(sys.argv[1])
dispatcher = Path(sys.argv[2]).resolve()
registry_path = Path(sys.argv[3])

payload = json.loads(registry_path.read_text(encoding="utf-8"))
entries = payload.get("entries")
if not isinstance(entries, dict) or not entries:
    raise SystemExit("expected non-empty entries map in exec registry")

invalid_entries: list[str] = []
for wrapper_rel, entry in sorted(entries.items()):
    wrapper_path = root / wrapper_rel
    if not wrapper_path.exists():
        invalid_entries.append(f"missing wrapper path: {wrapper_rel}")
        continue
    if not wrapper_path.is_symlink():
        invalid_entries.append(f"wrapper is not symlink: {wrapper_rel}")
        continue
    if wrapper_path.resolve() != dispatcher:
        invalid_entries.append(
            f"wrapper does not resolve to dispatcher: {wrapper_rel} -> {wrapper_path.resolve()}"
        )

    if not isinstance(entry, dict):
        invalid_entries.append(f"registry entry is not object: {wrapper_rel}")
        continue
    interpreter = entry.get("interpreter")
    target = entry.get("target")
    args_prefix = entry.get("args_prefix")
    passthrough = entry.get("passthrough")
    if interpreter not in {"python3", "bash"}:
        invalid_entries.append(f"invalid interpreter for {wrapper_rel}: {interpreter!r}")
    if not isinstance(target, str) or target.strip() == "":
        invalid_entries.append(f"invalid target for {wrapper_rel}: {target!r}")
    else:
        target_path = root / target
        if not target_path.exists():
            invalid_entries.append(f"target path missing for {wrapper_rel}: {target}")
    if not isinstance(args_prefix, list) or any(not isinstance(item, str) for item in args_prefix):
        invalid_entries.append(f"invalid args_prefix for {wrapper_rel}")
    if not isinstance(passthrough, bool):
        invalid_entries.append(f"invalid passthrough for {wrapper_rel}: {passthrough!r}")

if invalid_entries:
    raise SystemExit("\n".join(invalid_entries))

prefixes = ("$ROOT_DIR/", "$KAMN_ROOT/", "${ROOT_DIR}/", "${KAMN_ROOT}/")
remaining_tiny_wrappers: list[str] = []
for script in sorted((root / "scripts").rglob("*.sh")):
    if script.is_symlink():
        continue
    lines = script.read_text(encoding="utf-8").splitlines()
    exec_lines = [line.strip() for line in lines if line.strip().startswith("exec ")]
    if len(exec_lines) != 1:
        continue
    line = exec_lines[0]
    if line.endswith("\\"):
        continue
    tokens = shlex.split(line)
    if len(tokens) < 3 or tokens[0] != "exec" or tokens[1] not in {"python3", "bash"}:
        continue
    target = tokens[2]
    if not target.startswith(prefixes):
        continue
    rel = script.relative_to(root).as_posix()
    remaining_tiny_wrappers.append(rel)

if remaining_tiny_wrappers:
    message = ["expected zero remaining tiny exec wrappers after registry migration"]
    message.extend(remaining_tiny_wrappers[:80])
    if len(remaining_tiny_wrappers) > 80:
        message.append(f"... and {len(remaining_tiny_wrappers) - 80} more")
    raise SystemExit("\n".join(message))
PY

echo "exec wrapper registry migration tests passed."
