# FR authoring — session progress

**Session date:** 2026-05-15
**Standard:** every FR authored or revised to **10/10** with all open questions resolved upfront, failure modes inventory, OBS metrics enumerated, transaction shape explicit, and implementation skeleton compileable.

## §1 — Counts

| Module | FRs at 10/10 | Phase | Status |
|---|---:|---|---|
| AI Gateway | **22** | P0 | ✅ COMPLETE (slices 1-5) |
| OBS | **9** | P0 | ✅ COMPLETE (slices 1-3) |
| AUTH stub | **6** | P0 | ✅ COMPLETE (slice 1) |
| BRAIN auto-sync | **6** | P1 | partial (6 of 11) |
| SKILL | **2** | P1 | partial (2 of 9) |
| PROJ | **2** | P1 | partial (2 of 18) |
| **Total** | **47** | | |

Plus: **5 slice-1 AI Gateway FRs round-2 re-audited** from 9.4 average to 10/10 across the board.

## §2 — Per-module FR list

**AI Gateway (P0, 22 FRs):**
FR-AI-001 cost-ledger pre-call check · FR-AI-002 post-call reconcile · FR-AI-003 BRAIN audit-row bridge · FR-AI-004 cost-hold expiry cleanup · FR-AI-005 tenant-policy YAML loader · FR-AI-006 model-alias resolution · FR-AI-007 provider cost-table loader · FR-AI-008 multi-provider router · FR-AI-009 circuit breaker · FR-AI-010 streaming SSE · FR-AI-011 Presidio EN PII redaction · FR-AI-012 VN-PII plugin · FR-AI-013 VN-PII recall ≥99% CI gate · FR-AI-014 persona-version stamping · FR-AI-015 ZDR enforcement · FR-AI-016 residency pinning · FR-AI-017 per-tenant cache · FR-AI-018 cross-tenant cache leak property-test · FR-AI-019 BGE-M3 embeddings · FR-AI-020 BGE-rerank · FR-AI-021 operator CLI · FR-AI-022 OTel trace emission

**OBS (P0, 9 FRs):**
FR-OBS-001 OTel collector · FR-OBS-002 tenant-aware Grafana proxy · FR-OBS-003 RED metrics · FR-OBS-004 LangSmith integration · FR-OBS-005 TraceContext correlation · FR-OBS-006 tail sampling · FR-OBS-007 AlertManager → CUO triage routing · FR-OBS-008 compliance view scoping · FR-OBS-009 chain-of-custody manifest

**AUTH stub (P0, 6 FRs):**
FR-AUTH-001 tenant create · FR-AUTH-002 subject create · FR-AUTH-003 RLS enforcement · FR-AUTH-004 JWT + JWKS · FR-AUTH-005 admin REST · FR-AUTH-006 bootstrap CLI

**BRAIN auto-sync (P1, 6 of 11 FRs):**
FR-BRAIN-101 Layer 2 ingest pipeline · FR-BRAIN-102 Layer 2 rebuild CI gate · FR-BRAIN-103 multi-device sync · FR-BRAIN-104 Tauri app · FR-BRAIN-106 sync_class enforcement · FR-BRAIN-108 search API
**Pending:** FR-BRAIN-105 conflict UI · FR-BRAIN-107 cross-tenant merge · FR-BRAIN-109 scheduler · FR-BRAIN-110 doctor · FR-BRAIN-111 export

**SKILL (P1, 2 of 9 FRs):**
FR-SKILL-101 BRAIN integration · FR-SKILL-102 OCI registry
**Pending:** FR-SKILL-103..109

**PROJ (P1, 2 of 18 FRs):**
FR-PROJ-001 Issue + Cycle schema · FR-PROJ-002 BRAIN-anchored decisions
**Pending:** FR-PROJ-003..018

## §3 — Remaining for full P0 + user-priority P1

| Module | Remaining FRs | Effort |
|---|---:|---:|
| MCP Gateway (P0) | 8 | ~52h spec |
| CHAT (P0) | 12 | ~70h spec |
| BRAIN P1 | 5 | ~30h spec |
| SKILL P1 | 7 | ~45h spec |
| PROJ P1 | 16 | ~110h spec |
| **Total** | **48** | **~307h spec** |

## §4 — Quality artefacts produced

- 47 FR markdowns at 10/10 (each with frontmatter + §1–§10 sections)
- 11 audit summary files (slice 1, 2, 3 audits per module + cross-FR summaries)
- 5 per-FR audit files for AI Gateway slice 1 round-2 revisions
- 6 cross-FR consistency issues found and resolved (XFR-001..006)
- 80 failure-mode inventory rows
- 60+ OBS metrics enumerated

## §5 — Decision points open for user

1. **Accept the 47 FRs?** (transition each `draft → accepted`)
2. **Continue next batch?** (MCP + CHAT for P0, then BRAIN/SKILL/PROJ remainder for P1)
3. **Pivot to implementation?** (slice 1 of AI Gateway is the load-bearing first build target)

---

*End of session progress summary.*
