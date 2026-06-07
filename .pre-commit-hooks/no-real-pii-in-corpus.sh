#!/usr/bin/env bash
# FR-AI-013: reject VN PII corpus rows matching known internal customer PII.
set -euo pipefail

CORPUS="services/ai-gateway/pii/fixtures/vn_pii_200_samples.yaml"
KNOWN_PATTERNS_TABLE="${CYBEROS_KNOWN_CUSTOMER_PII_TABLE:-/opt/cyberos/pii_patterns.enc}"

if [[ ! -f "$CORPUS" ]]; then
    exit 0
fi

if [[ ! -f "$KNOWN_PATTERNS_TABLE" ]]; then
    echo "WARN: $KNOWN_PATTERNS_TABLE not present; skipping real-PII table check" >&2
    exit 0
fi

python3 services/ai-gateway/pii/scripts/check_no_real_pii.py \
    "$CORPUS" \
    "$KNOWN_PATTERNS_TABLE"
