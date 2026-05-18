---
template: deployment-checklist@1
title: <release-id> deployment readiness
release_id: <SemVer or tag>
release_candidate_sha: <full git SHA>
target_environment: production    # staging | canary | production
deploy_window: { start: 2026-MM-DDTHH:MM:SS+07:00, end: 2026-MM-DDTHH:MM:SS+07:00 }
provenance: { source_path: ./release-notes.md, source_hash: sha256:<hash> }
deploy_owner: @<owner>
change_ticket_id: <ticket ref>
progressive_delivery: canary    # canary | blue_green | feature_flag | rolling | direct
---

# <release-id> deployment readiness

| # | Item | Status | Evidence |
|---|---|---|---|
| DEP-001 | All DoDs met | ⏳ | ./dor-dod.md |
| DEP-002 | Release notes drafted | ⏳ | ./release-notes.md |
| DEP-003 | Rollback plan documented + rehearsed | ⏳ | <runbook ref> |
| DEP-004 | Feature flags configured | ⏳ | <flag refs> |
| DEP-005 | Database migrations rehearsed | ⏳ | <rehearsal log> |
| DEP-006 | Monitoring + alerts in place | ⏳ | <dashboards> |
| DEP-007 | On-call rota notified | ⏳ | <invite> |
| DEP-008 | Security scans clean | ⏳ | <SAST + SCA + secret-scan reports> |
| DEP-009 | Change ticket approved | ⏳ | <ticket> |
| DEP-010 | SBOM published | ⏳ | <sbom path or URL> |
| DEP-011 | DORA baseline captured | ⏳ | freq=<>, lead=<>, CFR=<>, MTTR=<> |
| DEP-012 | Signed artefacts | ⏳ | <signature attestation> |

<!-- ## Conditional rows (uncomment per COND-001..006) -->
<!-- DEP-013 Customer-comms drafted        — production -->
<!-- DEP-014 Deprecation timeline          — breaking change -->
<!-- DEP-015 DBA approved migration plan   — large migration -->
<!-- DEP-016 Compliance sign-off captured  — regulated path -->
<!-- DEP-017 Canary success/failure + auto-rollback wired  — canary -->
<!-- DEP-018 Model card + eval + rollback SHA  — AI-model update -->
