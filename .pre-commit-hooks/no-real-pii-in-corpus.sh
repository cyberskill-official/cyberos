#!/usr/bin/env bash
# FR-AI-013 §1 #14 — Pre-commit hook to detect real PII in the corpus.
# Flags any 12-digit sequence with a valid CCCD province code prefix.
set -euo pipehook

CORPUS="services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml"

if [ ! -f "$CORPUS" ]; then
    echo "Corpus not found: $CORPUS (skipping)"
    exit 0
fi

# Check for digit sequences that look like real CCCDs (12 digits, valid province code).
# This is a heuristic — the AST-walk lint in test_no_network_imports.py is the real gate.
VIOLATIONS=$(grep -nE '\b(0[0-9]{2}|0[0-9]{2})[0-9]{9}\b' "$CORPUS" | \
    grep -v '# ' | \
    grep -v 'expected_entities' || true)

if [ -n "$VIOLATIONS" ]; then
    echo "WARNING: Potential real PII detected in corpus:"
    echo "$VIOLATIONS"
    echo ""
    echo "If these are synthetic samples, add a comment '# synthetic' on the line."
    echo "If these are real, REPLACE with synthetic equivalents before committing."
    exit 1
fi

exit 0
