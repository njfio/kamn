#!/usr/bin/env bash
set -euo pipefail

if [ "${KAMN_KOLME_LOCAL_HEAVY:-0}" != "1" ]; then
  echo "run mode requires explicit local-only opt-in: KAMN_KOLME_LOCAL_HEAVY=1" >&2
  exit 1
fi
