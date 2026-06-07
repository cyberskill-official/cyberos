#!/usr/bin/env bash
# FR-CHAT-001 §5 — run all bash-level tests for the CHAT service.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

PASS=0
FAIL=0
FAILED=()

for test in \
    "$SCRIPT_DIR/pinned_commit_test.sh" \
    "$SCRIPT_DIR/license_drift_test.sh" \
    "$SCRIPT_DIR/patch_apply_test.sh" \
    "$SCRIPT_DIR/workflows_present_test.sh" \
    "$SCRIPT_DIR/../plugins/cyberos-authbridge/tests/authbridge_shape_test.sh"
do
    name=$(basename "$test")
    echo ""
    echo "============================================================"
    echo "  Running: $name"
    echo "============================================================"
    if bash "$test"; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        FAILED+=("$name")
    fi
done

echo ""
echo "============================================================"
echo "  Running: test_cyberos_chat.py"
echo "============================================================"
if python -m pytest "$SCRIPT_DIR/test_cyberos_chat.py"; then
    PASS=$((PASS + 1))
else
    FAIL=$((FAIL + 1))
    FAILED+=("test_cyberos_chat.py")
fi

echo ""
echo "============================================================"
echo "CHAT test summary"
echo "============================================================"
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
if [[ $FAIL -gt 0 ]]; then
    echo ""
    echo "Failed tests:"
    for f in "${FAILED[@]}"; do echo "  - $f"; done
    exit 1
fi
echo ""
echo "✓ All CHAT tests pass."
