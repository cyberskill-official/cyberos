---
template: threat-model@1
title: <System> — Threat Model
system_under_threat: <System / product / service name>
tm_version: 1.0.0
modelled_at: 2026-MM-DDTHH:MM:SS+07:00
modelled_by:
  - { handle: "@<ar>",    role: "Architect" }
  - { handle: "@<sec>",   role: "SEC" }
linked_srs: ./srs.md
linked_adrs: [./adrs/ADR-0001.md, ./adrs/ADR-0042.md]
provenance: { source_path: ./srs.md, source_hash: sha256:<hash> }
asvs_level: L2    # L1 | L2 | L3
next_review_date: 2026-MM-DD    # ≤180 days from modelled_at
---

# <System> — Threat Model

## 1. System Overview
<1-2 paragraphs + reference to context diagram asset.>

## 2. Trust Boundaries
| # | Boundary | Inside privilege | Outside privilege |
|---|---|---|---|

## 3. Data Flow Diagram
See `./assets/dfd.png` (or `./assets/dfd.mermaid`).

## 4. Threats by STRIDE Category

### 4.1 Spoofing
| id | Threat | Mitigation | Owner |
|---|---|---|---|

### 4.2 Tampering
### 4.3 Repudiation
### 4.4 Information Disclosure
### 4.5 Denial of Service
### 4.6 Elevation of Privilege

## 5. OWASP Top 10:2025 Coverage
| risk | Treatment | STRIDE refs |
|---|---|---|
| A01 Broken Access Control | <treatment> | STRIDE-E threats |
| A02 Security Misconfiguration | ... | STRIDE-T |
| A03 Software Supply Chain Failures | ... | SBOM + provenance |
| A04 Cryptographic Failures | ... | ... |
| A05 Injection | ... | STRIDE-T |
| A06 Insecure Design | ... | ... |
| A07 Authentication Failures | ... | STRIDE-S |
| A08 Software & Data Integrity Failures | ... | ... |
| A09 Security Logging & Alerting Failures | ... | STRIDE-R |
| A10 Mishandling of Exceptional Conditions | ... | ... |

## 6. OWASP ASVS Controls Mapping
Per the declared `asvs_level`.
| ASVS control | Status (implemented / compensated / accepted-risk) | Evidence |
|---|---|---|

## 7. Residual Risk Register
| Threat ref | Acceptance rationale | Owner | Review date |
|---|---|---|---|

## 8. Mitigations and Linked ADRs
| Mitigation | Linked ADR |
|---|---|

<!-- ## 9. Privacy Threat Analysis (LINDDUN)    — when system processes personal data -->
<!-- ## 10. ML-Specific Threats                 — when system uses AI/ML -->
<!-- ## 11. API-Specific Threats                — when system exposes a public API -->
