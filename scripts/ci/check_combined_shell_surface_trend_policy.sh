#!/usr/bin/env bash
set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/scripts/lib/common.sh"

exec python3 "$KAMN_ROOT/scripts/ci/check_combined_shell_surface_trend_policy.py" "$@"
