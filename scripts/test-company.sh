#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
npm --prefix e2e ci
if command -v xvfb-run >/dev/null 2>&1; then
  xvfb-run -a npm --prefix e2e run test:company
else
  npm --prefix e2e run test:company
fi
