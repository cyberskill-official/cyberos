# VN PII 200-Sample Fixture — Provenance & Maintenance

## Overview

This fixture contains 200 samples (170 positive + 30 negative) for testing the VN PII recognizers (TASK-AI-012) and the recall-floor CI gate (TASK-AI-013).

## Sample Distribution

| Entity Type    | Count | Source |
|----------------|-------|--------|
| VN_CCCD        | 30    | Synthetic |
| VN_MST         | 30    | Synthetic |
| VN_PHONE       | 30    | Synthetic |
| VN_NDD         | 20    | Synthetic |
| VN_ADDRESS     | 30    | Synthetic |
| VN_BANK_ACCOUNT| 30    | Synthetic |
| Negative       | 30    | Synthetic |
| **Total**      | **200** | |

## Provenance

All samples are **synthetic** — no real customer data is used. Province codes are real (from Vietnam's General Statistics Office) but digit sequences are fabricated to be format-valid without matching any real identity.

## Regeneration Schedule

Regenerated quarterly: Jan 1, Apr 1, Jul 1, Oct 1.

## Curator

Stephen Cheng (CEO, CyberSkill)

## Last Updated

2026-05-21
