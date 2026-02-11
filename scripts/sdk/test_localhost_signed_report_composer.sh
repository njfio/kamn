#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

python3 "$ROOT_DIR/scripts/sdk/test_localhost_signed_report_composer.py"

echo "localhost signed report composer helper tests passed."
