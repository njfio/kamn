#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMMON_LIB="$ROOT_DIR/scripts/lib/common.sh"
HELPER_SCRIPT="$ROOT_DIR/scripts/lib/write_json_file.sh"

if [ ! -f "$COMMON_LIB" ]; then
  echo "expected shared common library to exist: $COMMON_LIB" >&2
  exit 1
fi

if ! grep -q '^write_json_file()' "$COMMON_LIB"; then
  echo "expected write_json_file() helper in common library" >&2
  exit 1
fi

if ! grep -q '^write_json_object()' "$COMMON_LIB"; then
  echo "expected write_json_object() helper in common library" >&2
  exit 1
fi

if ! grep -q '^write_decision_json()' "$COMMON_LIB"; then
  echo "expected write_decision_json() helper in common library" >&2
  exit 1
fi

if [ ! -x "$HELPER_SCRIPT" ]; then
  echo "expected JSON write helper command to be executable: $HELPER_SCRIPT" >&2
  exit 1
fi

migrated_script_count="$(rg -l 'scripts/lib/write_json_file.sh' "$ROOT_DIR/scripts" -g '*.sh' | wc -l | tr -d ' ')"
if [ "$migrated_script_count" -lt 80 ]; then
  echo "expected at least 80 migrated scripts using write_json_file helper, found: $migrated_script_count" >&2
  exit 1
fi

legacy_rootdir_json_count="$(python3 - "$ROOT_DIR/scripts" <<'PY'
import re
import sys
from pathlib import Path

scripts_root = Path(sys.argv[1])
legacy_count = 0

for script_path in scripts_root.rglob("*.sh"):
    text = script_path.read_text(encoding="utf-8", errors="replace")
    if "ROOT_DIR=" not in text:
        continue
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i]
        match = re.search(r"<<-?\s*['\"]?([A-Za-z_][A-Za-z0-9_]*)['\"]?", line)
        if not match:
            i += 1
            continue
        delimiter = match.group(1)
        if not re.search(r"\bcat\b", line) or ">" not in line:
            i += 1
            continue
        j = i + 1
        block = []
        while j < len(lines) and lines[j].strip() != delimiter:
            block.append(lines[j])
            j += 1
        content = "\n".join(block).strip()
        if content.startswith("{") and ":" in content:
            legacy_count += 1
        i = j + 1

print(legacy_count)
PY
)"

if [ "$legacy_rootdir_json_count" -ne 0 ]; then
  echo "expected zero remaining ROOT_DIR-based cat-heredoc JSON writers, found: $legacy_rootdir_json_count" >&2
  exit 1
fi

echo "JSON write helper migration contract tests passed."
