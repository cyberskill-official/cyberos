#!/usr/bin/env bash
# install.sh — dev install of cuo module
set -euo pipefail
cd "$(dirname "$0")/.."
pip install -e .
echo "✓ cyberos-cuo installed; try:"
echo "    cyberos-cuo catalog"
echo "    cyberos-cuo route 'Validate MST 0312345678'"
