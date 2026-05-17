---
template: test-strategy@1
title: <Project / release> — Test Strategy
strategy_version: 1.0.0
linked_srs: ./srs.md
risk_class: medium    # low | medium | high
provenance: { source_path: ./srs.md, source_hash: sha256:<hash> }
effective_date: 2026-MM-DD
qa_owner: @<qa>
---

# <Project / release> — Test Strategy

## 1. Scope
<In / out of scope for this release.>

## 2. Risk-Based Test Priorities
| risk | test approach | priority |
|---|---|---|

## 3. Test Levels
### 3.1 Unit
### 3.2 Integration
### 3.3 System
### 3.4 UAT

## 4. Test Types
### 4.1 Functional
### 4.2 Performance — tool, target percentile, soak duration
### 4.3 Security — OWASP Top 10:2025 coverage, ZAP/Burp policy
### 4.4 Accessibility — WCAG 2.2 level + axe/Pa11y
### 4.5 Regression

## 5. Environments and Data
| env | refresh | data source | anonymisation |
|---|---|---|---|

## 6. Tooling
| tool | version | scope |
|---|---|---|

## 7. Entry Criteria
- <≥3 measurable bullets>

## 8. Exit Criteria
- <≥3 measurable bullets>

## 9. Defect Management
| severity | response SLA | resolution SLA |
|---|---|---|

## 10. Metrics
- defect density, defect leakage, automation coverage target, MTTD/MTTR for prod-found defects.

<!-- ## 11. Threat-Led Pen-Test Plan    — required when risk_class: high -->
<!-- ## 12. Data Privacy Test Cases     — when personal data -->
<!-- ## 13. AI-Specific Test Cases      — when AI-driven -->
