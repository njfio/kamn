#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/common.sh"

usage() {
  cat >&2 <<'USAGE'
Usage: write_json_file.sh <output-json-path>
Reads JSON content from stdin and writes it to <output-json-path>.
USAGE
}

if [ "$#" -ne 1 ]; then
  usage
  exit 1
fi

write_json_file "$1"
