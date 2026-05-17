# CyberOS — Generated FR Reports (combined)

_Last generated 2026-05-17 — 241 FRs. Refresh after FR-CUO-106 lands (and any other adds) by re-running the regen scripts; this file consolidates four previously-separate reports per the 2026-05-18 doc-consolidation directive._

This document combines what previously lived in 4 separate files (now deleted):

| Section | Was | Purpose |
|---|---|---|
| §1 | `CONTRACT_VERIFICATION_REPORT.md` | Cross-FR API endpoint consistency scan |
| §2 | `IMPLEMENTATION_ORDER.md` | Topological build sequence (13 layers) |
| §3 | `SPRINT_PLAN.md` | Effort rollup by module + slice + capacity math |
| §4 | `MIGRATION_AUDIT.md` | Per-module SQL migration sequence audit |

All four are **generated artefacts** — the regen scripts read FR frontmatter and rebuild the corpus. Hand-edit the FRs, not this file. After running regens, replace the section bodies in-place; section ordering is normative.

For the narrative backlog see [`BACKLOG.md`](BACKLOG.md). For FR-authoring discipline see [`AUTHORING.md`](AUTHORING.md). For Vietnamese-regulatory terminology see [`VN_GLOSSARY.md`](VN_GLOSSARY.md).

---

# §1 — Contract verification report


_Generated 2026-05-17 — 241 FRs scanned._

## Summary

- Endpoints declared (in §3 API contract): **261**
- Endpoint references across all FR text: **595**
- Endpoints with multiple declarers (potential conflict): **0**
- Orphan endpoint references (not declared anywhere): **333**

## Endpoints declared (per FR)

- **FR-AUTH-105**: POST /v1/auth/mfa/recovery-codes/regen
- **FR-CRM-004**: POST /v1/crm/deals/{}/convert-to-engagement
- **FR-CRM-005**: POST /v1/crm/next-action, POST /v1/crm/next-action/{}/dismiss, POST /v1/crm/next-action/{}/execute
- **FR-CRM-006**: GET /v1/crm/contacts/{}/score-history, GET /v1/crm/scoring/weights, POST /v1/crm/contacts/{}/rescore, PUT /v1/crm/scoring/weights
- **FR-CRM-007**: GET /v1/crm/win-loss/drafts, POST /v1/crm/win-loss/drafts/{}/approve, POST /v1/crm/win-loss/drafts/{}/dismiss
- **FR-CRM-008**: GET /v1/crm/accounts/{}/mst-validation, POST /v1/crm/accounts/{}/validate-mst
- **FR-CRM-009**: GET /v1/crm/bank-config, POST /v1/crm/skill/vn-bank-transfer, PUT /v1/crm/bank-config
- **FR-CRM-010**: GET /v1/crm/skill/vn-vat-invoice/emissions/{}, POST /v1/crm/skill/vn-vat-invoice
- **FR-CUO-104**: POST /v1/cuo/chains
- **FR-DOC-002**: GET /v1/doc/qtsp/signatures/{}, POST /v1/doc/qtsp/sign, PUT /v1/doc/qtsp/creds
- **FR-DOC-004**: GET /v1/doc/vn-ca/signatures/{}, POST /v1/doc/vn-ca/sign, POST /v1/doc/vn-ca/vneid-link, PUT /v1/doc/vn-ca/creds
- **FR-DOC-005**: GET /v1/doc/signing-workflows/{}, POST /v1/doc/documents/{}/signing-workflows, POST /v1/doc/signers/{}/decline, POST /v1/doc/signers/{}/sign, POST /v1/doc/signing-workflows/{}/withdraw
- **FR-DOC-006**: GET /v1/doc/documents/{}/verifications, POST /v1/doc/documents/{}/verify/complete, POST /v1/doc/documents/{}/verify/start
- **FR-DOC-008**: DELETE /v1/doc/documents/{}/snooze-alerts, GET /v1/doc/expiry-alerts, POST /v1/doc/documents/{}/snooze-alerts, POST /v1/doc/expiry-scan
- **FR-DOC-010**: GET /v1/doc/import/jobs/{}, POST /v1/doc/import/{}/start, PUT /v1/doc/third-party-creds
- **FR-DOC-011**: GET /v1/doc/documents/{}/ltv/operations, POST /v1/doc/documents/{}/ltv/extend, POST /v1/doc/documents/{}/ltv/restamp
- **FR-EMAIL-002**: POST /v1/email/auth
- **FR-EMAIL-003**: POST /v1/email/threads/{}/comments
- **FR-EMAIL-004**: POST /v1/admin/tenants/{}/email/bimi-enable, POST /v1/admin/tenants/{}/email/dns-setup, POST /v1/admin/tenants/{}/email/dns-verify
- **FR-EMAIL-005**: GET /v1/email/camel/audit-log, POST /v1/email/camel/execute, PUT /v1/email/camel/trust-list
- **FR-EMAIL-006**: POST /v1/email/tracked-domains
- **FR-EMAIL-007**: GET /v1/email/messages/{}/converted-issues, POST /v1/email/messages/{}/convert-to-issue
- **FR-EMAIL-008**: GET /v1/email/genie/sessions, GET /v1/email/genie/sessions/{}, POST /v1/email/genie/actions/{}/approve, POST /v1/email/genie/actions/{}/dismiss, PUT /v1/email/genie/config
- **FR-EMAIL-009**: GET /v1/email/outbound, POST /v1/admin/email/suppression/unsuppress, POST /v1/email/outbound/compose, POST /v1/email/outbound/send
- **FR-EMAIL-011**: GET /v1/email/dsar/jobs/{}, POST /v1/email/dsar/export
- **FR-ESOP-001**: POST /v1/esop/grants
- **FR-ESOP-003**: POST /v1/esop/valuations
- **FR-ESOP-004**: POST /v1/esop/puts
- **FR-ESOP-006**: POST /v1/esop/ma-events
- **FR-ESOP-007**: GET /v1/esop/members/{}/dashboard
- **FR-HR-002**: GET /v1/hr/members/{}/contract-history, PUT /v1/hr/members/{}/contract
- **FR-HR-003**: POST /v1/hr/members/{}/cccd-consent
- **FR-HR-004**: GET /v1/hr/members/{}/leave-balance, POST /v1/hr/leave-requests, POST /v1/hr/leave-requests/{}/approve, POST /v1/hr/leave-requests/{}/cancel, POST /v1/hr/leave-requests/{}/reject
- **FR-HR-005**: GET /v1/hr/policy
- **FR-HR-006**: POST /v1/hr/accrual/corrections
- **FR-HR-009**: GET /v1/hr/terminations/{}, POST /v1/hr/terminations, POST /v1/hr/terminations/{}/ceo-sign, POST /v1/hr/terminations/{}/cfo-sign, POST /v1/hr/terminations/{}/dispute
- **FR-INV-001**: GET /v1/inv/invoices, GET /v1/inv/invoices/{}, POST /v1/inv/invoices/draft, POST /v1/inv/invoices/{}/approve, POST /v1/inv/invoices/{}/lines/correction, POST /v1/inv/invoices/{}/send, POST /v1/inv/invoices/{}/void, POST /v1/inv/invoices/{}/write-off
- **FR-INV-002**: GET /v1/inv/fx/convert, GET /v1/inv/fx/rates, POST /v1/admin/inv/fx/override
- **FR-INV-007**: GET /v1/inv/hoadon/{}, POST /v1/inv/hoadon/emit, POST /v1/inv/hoadon/{}/resubmit
- **FR-INV-008**: GET /v1/inv/cancellation-forms, GET /v1/inv/hoadon/{}/cancellation, POST /v1/inv/hoadon/{}/cancel
- **FR-INV-009**: GET /v1/inv/reports/aging, POST /v1/inv/reports/aging
- **FR-INV-010**: GET /v1/inv/dunning/drafts, POST /v1/inv/dunning/drafts/{}/approve, POST /v1/inv/dunning/drafts/{}/dismiss, POST /v1/inv/dunning/scan
- **FR-INV-011**: GET /v1/inv/recognition/journal-entries, GET /v1/inv/recognition/schedules/{}, GET /v1/inv/recognition/snapshots/{}, POST /v1/inv/recognition/rollforward, POST /v1/inv/recognition/schedules
- **FR-KB-002**: GET /v1/kb/docs/{}/render, POST /v1/kb/docs/{}/render
- **FR-KB-003**: GET /v1/kb/docs/{}, POST /v1/kb/docs/{}/share-links, POST /v1/kb/share-links/{}/revoke, PUT /v1/kb/docs/{}/visibility
- **FR-KB-004**: POST /v1/kb/search/lexical
- **FR-KB-005**: POST /v1/kb/search/semantic
- **FR-KB-006**: POST /v1/kb/search/rerank
- **FR-KB-007**: POST /v1/kb/docs/{}/ask
- **FR-KB-008**: GET /v1/kb/runbooks/match, PUT /v1/kb/docs/{}/runbook-tags
- **FR-KB-009**: PUT /v1/kb/docs/{}/translation
- **FR-LEARN-001**: POST /v1/learn/members/{}/mastery
- **FR-LEARN-004**: POST /v1/learn/councils, POST /v1/learn/councils/{}/scores
- **FR-LEARN-005**: GET /v1/learn/councils/{}/disclosure
- **FR-MCP-003**: POST /v1/mcp/naming/validate
- **FR-MCP-006**: GET /v1/admin/tenants/{}/mcp/gating-decisions, POST /v1/admin/tenants/{}/mcp/gating-policy, POST /v1/admin/tenants/{}/mcp/gating-policy/activate, POST /v1/mcp/tools/{}/confirm
- **FR-MCP-007**: GET /v1/mcp/tasks, GET /v1/mcp/tasks/{}, POST /v1/mcp/tasks/{}/cancel, POST /v1/mcp/tools/{}/call
- **FR-MCP-008**: GET /v1/mcp/elicitations, POST /v1/mcp/elicitations/{}/cancel, POST /v1/mcp/elicitations/{}/respond
- **FR-OKR-003**: POST /v1/okr/krs/{}/custom-sql/ceo-sign, POST /v1/okr/krs/{}/custom-sql/cfo-sign, POST /v1/okr/krs/{}/custom-sql/request
- **FR-OKR-005**: POST /v1/okr/krs/{}/checkins
- **FR-OKR-006**: GET /v1/okr/digest/runs, POST /v1/okr/digest/trigger, PUT /v1/okr/digest/recipients/{}
- **FR-PORTAL-001**: GET /v1/portal/views/{}, GET /v1/portal/views/{}/export, GET /v1/portal/views/{}/{}, POST /v1/portal/views/{}/search
- **FR-PORTAL-002**: GET /v1/admin/tenants/{}/brand-pack/{}/export, POST /v1/admin/tenants/{}/brand-pack, POST /v1/admin/tenants/{}/brand-pack/rollback, POST /v1/admin/tenants/{}/brand-pack/{}/activate, POST /v1/admin/tenants/{}/cname, POST /v1/admin/tenants/{}/cname/{}/verify
- **FR-PORTAL-003**: GET /v1/portal/sign-in, PATCH /v1/admin/engagements/{}/idp, POST /v1/admin/engagements/{}/idp, POST /v1/admin/engagements/{}/idp/groups-map, POST /v1/admin/engagements/{}/scim-token/rotate
- **FR-PORTAL-004**: GET /v1/admin/tenants/{}/deprovision-log, POST /v1/admin/engagements/{}/subjects/{}/restore
- **FR-PORTAL-005**: GET /v1/portal/genie/sessions, GET /v1/portal/genie/sessions/{}/messages, POST /v1/portal/genie/query, POST /v1/portal/genie/sessions/{}/archive
- **FR-PORTAL-006**: GET /v1/portal/workflows, GET /v1/portal/workflows/{}, POST /v1/admin/tenants/{}/workflow-routes, POST /v1/portal/workflows/submit, POST /v1/portal/workflows/{}/reopen, POST /v1/portal/workflows/{}/reply
- **FR-PORTAL-007**: GET /v1/portal/pwa/subscriptions, PATCH /v1/portal/pwa/preferences, POST /v1/portal/pwa/subscribe, POST /v1/portal/pwa/unsubscribe
- **FR-PORTAL-008**: GET /v1/admin/tenants/{}/dsar, GET /v1/portal/dsar/{}, POST /v1/admin/dsar/{}/deny, POST /v1/portal/dsar/request
- **FR-RES-002**: POST /v1/res/allocations/propose
- **FR-RES-005**: POST /v1/res/ot-consent
- **FR-REW-001**: POST /v1/rew/comp
- **FR-REW-002**: GET /v1/rew/params/tax_bracket
- **FR-REW-003**: POST /v1/rew/p1-demotion-consents
- **FR-REW-006**: GET /v1/rew/payslips/{}/pdf, POST /v1/rew/payslips/{}/render
- **FR-REW-007**: POST /v1/rew/bp/credits
- **FR-REW-008**: POST /v1/rew/p3-distributions
- **FR-SKILL-201**: POST /v1/skill/oci/push
- **FR-TEN-003**: GET /v1/admin/tenants/{}/billing, POST /v1/admin/tenants/{}/billing/refund
- **FR-TEN-005**: GET /v1/admin/packs/catalog, GET /v1/admin/tenants/{}/packs, POST /v1/admin/tenants/{}/packs/install, POST /v1/admin/tenants/{}/packs/{}/override, POST /v1/admin/tenants/{}/packs/{}/reinstall, POST /v1/admin/tenants/{}/packs/{}/uninstall
- **FR-TEN-101**: GET /v1/signup/oidc-callback, GET /v1/signup/slug-available, POST /v1/signup/complete, POST /v1/signup/payment-intent, POST /v1/signup/start, POST /v1/signup/verify-otp
- **FR-TEN-102**: GET /v1/admin/tenants/{}/vnd/invoices, GET /v1/admin/tenants/{}/vnd/invoices/{}, GET /v1/signup/vnd/token-bind-return, POST /v1/admin/tenants/{}/vnd/refund, POST /v1/admin/tenants/{}/vnd/token/revoke, POST /v1/signup/vnd/token-bind-start
- **FR-TEN-105**: GET /v1/admin/tenants/{}/bundle, GET /v1/admin/tenants/{}/bundle/{}/download, GET /v1/admin/tenants/{}/bundle/{}/verify, POST /v1/admin/tenants/{}/bundle/export
- **FR-TEN-106**: GET /v1/admin/permanent-delete/{}/verify, POST /v1/admin/permanent-delete/{}/cancel, POST /v1/admin/permanent-delete/{}/execute, POST /v1/admin/permanent-delete/{}/retry-cascade/{}, POST /v1/admin/permanent-delete/{}/sign-clo, POST /v1/admin/permanent-delete/{}/sign-cso, POST /v1/admin/tenants/{}/permanent-delete/initiate
- **FR-TEN-107**: GET /v1/ten/admin/audit-events
- **FR-TEN-202**: POST /v1/ten/hostile-overrides
- **FR-TIME-002**: GET /v1/time/timer/current, POST /v1/time/timer/abandon, POST /v1/time/timer/heartbeat, POST /v1/time/timer/pause, POST /v1/time/timer/resume, POST /v1/time/timer/start, POST /v1/time/timer/stop
- **FR-TIME-003**: GET /v1/time/entries/manual/pending-approvals, POST /v1/time/entries/manual
- **FR-TIME-004**: GET /v1/time/proposals, POST /v1/time/proposals/{}/accept, POST /v1/time/proposals/{}/reject
- **FR-TIME-005**: PATCH /v1/admin/tenants/{}, PATCH /v1/engagements/{}, PATCH /v1/projects/{}
- **FR-TIME-006**: GET /v1/time/timesheets/mine, GET /v1/time/timesheets/pending, GET /v1/time/timesheets/{}/diff, POST /v1/time/timesheets/bulk-approve, POST /v1/time/timesheets/{}/approve, POST /v1/time/timesheets/{}/reject, POST /v1/time/timesheets/{}/submit
- **FR-TIME-007**: GET /v1/time/vn-ot/status, POST /v1/admin/members/{}/vn-ot-approval
- **FR-TIME-008**: GET /v1/time/expenses, POST /v1/admin/engagements/{}/expense-policy, POST /v1/time/expenses/upload, POST /v1/time/expenses/{}/approve, POST /v1/time/expenses/{}/attach-to-invoice, POST /v1/time/expenses/{}/confirm, POST /v1/time/expenses/{}/reject
- **FR-TIME-009**: POST /v1/time/rollup

## Orphan endpoint references

Endpoint paths referenced in FR text but not declared in any FR's §3 API contract. May be intentional (external API, future-FR, internal-only) — review below:

- `FR-AI-001` references `POST /v1/chat/completions` — no FR declares this in §3.
- `FR-AI-104` references `GET /v1/ai/vn-providers/health` — no FR declares this in §3.
- `FR-AI-104` references `PUT /v1/ai/vn-providers/{}/creds` — no FR declares this in §3.
- `FR-AUTH-001` references `PATCH /v1/admin/tenants/<id>` — no FR declares this in §3.
- `FR-AUTH-001` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-AUTH-002` references `POST /v1/admin/subjects` — no FR declares this in §3.
- `FR-AUTH-003` references `POST /v1/admin/subjects` — no FR declares this in §3.
- `FR-AUTH-004` references `POST /v1/auth/token` — no FR declares this in §3.
- `FR-AUTH-005` references `GET /v1/admin/subjects` — no FR declares this in §3.
- `FR-AUTH-005` references `GET /v1/admin/tenants` — no FR declares this in §3.
- `FR-AUTH-005` references `POST /v1/admin/subjects/abc-id/revoke` — no FR declares this in §3.
- `FR-AUTH-005` references `POST /v1/admin/subjects/{}/revoke` — no FR declares this in §3.
- `FR-AUTH-005` references `POST /v1/admin/subjects/{}/unrevoke` — no FR declares this in §3.
- `FR-AUTH-006` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-AUTH-101` references `DELETE /v1/admin/subjects/{}/roles` — no FR declares this in §3.
- `FR-AUTH-101` references `DELETE /v1/admin/subjects/{}/roles/{}` — no FR declares this in §3.
- `FR-AUTH-101` references `GET /v1/admin/roles` — no FR declares this in §3.
- `FR-AUTH-101` references `GET /v1/admin/roles**` — no FR declares this in §3.
- `FR-AUTH-101` references `POST /v1/admin/subjects/{}/roles` — no FR declares this in §3.
- `FR-AUTH-102` references `DELETE /v1/auth/mfa/factors/{}` — no FR declares this in §3.
- `FR-AUTH-102` references `GET /v1/auth/mfa/factors` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/challenges` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/totp/enrol` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/totp/enrol/finish` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/webauthn/enrol/begin` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/factors/webauthn/enrol/finish` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/recovery-codes/consume` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/unlock` — no FR declares this in §3.
- `FR-AUTH-102` references `POST /v1/auth/mfa/verify` — no FR declares this in §3.
- `FR-AUTH-103` references `GET /v1/auth/saml/idp-configs/{}/sp-metadata` — no FR declares this in §3.
- `FR-AUTH-103` references `GET /v1/auth/saml/initiate` — no FR declares this in §3.
- `FR-AUTH-103` references `POST /v1/auth/saml/acs` — no FR declares this in §3.
- `FR-AUTH-103` references `POST /v1/auth/saml/idp-configs` — no FR declares this in §3.
- `FR-AUTH-104` references `GET /v1/auth/oidc/callback` — no FR declares this in §3.
- `FR-AUTH-104` references `GET /v1/auth/oidc/initiate` — no FR declares this in §3.
- `FR-AUTH-104` references `PATCH /v1/auth/oidc/idp-configs/{}` — no FR declares this in §3.
- `FR-AUTH-104` references `POST /v1/auth/oidc/idp-configs` — no FR declares this in §3.
- `FR-AUTH-105` references `DELETE /v1/auth/passkey/factors/{}` — no FR declares this in §3.
- `FR-AUTH-105` references `GET /v1/auth/passkey/factors` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/autofill-options` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/downgrade-optout` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/enrol/begin` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/enrol/finish` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/login/begin` — no FR declares this in §3.
- `FR-AUTH-105` references `POST /v1/auth/passkey/login/finish` — no FR declares this in §3.
- `FR-AUTH-106` references `POST /v1/auth/login` — no FR declares this in §3.
- `FR-AUTH-106` references `POST /v1/auth/mfa/challenge/{}/verify` — no FR declares this in §3.
- `FR-AUTH-106` references `PUT /v1/admin/tenants/{}/travel-policy` — no FR declares this in §3.
- `FR-AUTH-107` references `POST /v1/admin/subjects/{}/password` — no FR declares this in §3.
- `FR-AUTH-107` references `POST /v1/auth/password/rotate` — no FR declares this in §3.
- `FR-AUTH-107` references `POST /v1/auth/signup` — no FR declares this in §3.
- `FR-AUTH-107` references `PUT /v1/admin/tenants/{}/hibp-policy` — no FR declares this in §3.
- `FR-AUTH-108` references `GET /v1/auth/lumi/verify` — no FR declares this in §3.
- `FR-AUTH-108` references `POST /v1/auth/lumi/issue` — no FR declares this in §3.
- `FR-AUTH-109` references `GET /v1/auth/migration/preview` — no FR declares this in §3.
- `FR-AUTH-109` references `GET /v1/auth/migration/refresh-events` — no FR declares this in §3.
- `FR-AUTH-109` references `POST /v1/auth/migration/extend-grace` — no FR declares this in §3.
- `FR-BRAIN-108` references `GET /v1/brain/search` — no FR declares this in §3.
- `FR-CRM-001` references `DELETE /v1/crm/contacts/{}/memberships/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/accounts/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/contacts/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/deals/{}` — no FR declares this in §3.
- `FR-CRM-001` references `GET /v1/crm/pipelines` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/accounts` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/accounts/{}` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/contacts` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/deals` — no FR declares this in §3.
- `FR-CRM-001` references `PATCH /v1/crm/deals/{}` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/accounts` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/contacts` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/contacts/{}/memberships` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/deals` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/deals/{}/stage` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/deals/{}/status` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/pipelines` — no FR declares this in §3.
- `FR-CRM-001` references `POST /v1/crm/pipelines/{}/stages` — no FR declares this in §3.
- `FR-CRM-002` references `GET /v1/crm/accounts/{}/activities` — no FR declares this in §3.
- `FR-CRM-002` references `GET /v1/crm/contacts/{}/activities` — no FR declares this in §3.
- `FR-CRM-002` references `POST /v1/crm/activities` — no FR declares this in §3.
- `FR-CRM-004` references `GET /v1/crm/deals/{}/conversion` — no FR declares this in §3.
- `FR-CUO-102` references `GET /v1/cuo/runs/{}/checkpoints` — no FR declares this in §3.
- `FR-CUO-103` references `GET /v1/cuo/runs/{}/trace` — no FR declares this in §3.
- `FR-CUO-103` references `POST /v1/cuo/trace/{}/replay` — no FR declares this in §3.
- `FR-CUO-104` references `GET /v1/cuo/chains/{}` — no FR declares this in §3.
- `FR-CUO-105` references `GET /v1/cuo/chains/{}/rollback-status` — no FR declares this in §3.
- `FR-CUO-105` references `POST /v1/cuo/chains/{}/rollback` — no FR declares this in §3.
- `FR-DOC-001` references `GET /v1/doc/documents` — no FR declares this in §3.
- `FR-DOC-001` references `GET /v1/doc/documents/{}` — no FR declares this in §3.
- `FR-DOC-001` references `PATCH /v1/doc/documents` — no FR declares this in §3.
- `FR-DOC-001` references `PATCH /v1/doc/documents/{}` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/archive` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/finalize` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/legal-hold` — no FR declares this in §3.
- `FR-DOC-001` references `POST /v1/doc/documents/{}/versions` — no FR declares this in §3.
- `FR-DOC-003` references `GET /v1/doc/aatl/signatures/{}` — no FR declares this in §3.
- `FR-DOC-003` references `POST /v1/doc/aatl/sign` — no FR declares this in §3.
- `FR-DOC-003` references `PUT /v1/doc/aatl/creds` — no FR declares this in §3.
- `FR-DOC-007` references `GET /v1/doc/documents/{}/lifecycle` — no FR declares this in §3.
- `FR-DOC-007` references `GET /v1/doc/documents/{}/parent-chain` — no FR declares this in §3.
- `FR-DOC-007` references `PUT /v1/doc/documents/{}/lifecycle` — no FR declares this in §3.
- `FR-DOC-009` references `GET /v1/doc/renewal-drafts` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/documents/{}/draft-renewal` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/renewal-drafts/{}/approve` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/renewal-drafts/{}/dismiss` — no FR declares this in §3.
- `FR-DOC-009` references `POST /v1/doc/renewal-drafts/{}/send` — no FR declares this in §3.
- `FR-DOC-010` references `GET /v1/doc/imports` — no FR declares this in §3.
- `FR-EMAIL-001` references `GET /v1/email/healthz` — no FR declares this in §3.
- `FR-EMAIL-001` references `GET /v1/email/messages` — no FR declares this in §3.
- `FR-EMAIL-001` references `GET /v1/email/messages/{}/status` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/assign` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/close` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/reopen` — no FR declares this in §3.
- `FR-EMAIL-003` references `POST /v1/email/threads/{}/snooze` — no FR declares this in §3.
- `FR-EMAIL-006` references `DELETE /v1/email/tracked-domains/{}` — no FR declares this in §3.
- `FR-EMAIL-006` references `GET /v1/email/tracked-domains` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/draft` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/cancel` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/dispatch` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/sign-am` — no FR declares this in §3.
- `FR-EMAIL-010` references `POST /v1/email/bulk/{}/sign-cfo` — no FR declares this in §3.
- `FR-ESOP-001` references `GET /v1/esop/grants/{}` — no FR declares this in §3.
- `FR-ESOP-001` references `POST /v1/esop/grants/{}/cancel` — no FR declares this in §3.
- `FR-ESOP-001` references `POST /v1/esop/grants/{}/ceo-sign` — no FR declares this in §3.
- `FR-ESOP-001` references `POST /v1/esop/grants/{}/member-sign` — no FR declares this in §3.
- `FR-ESOP-002` references `GET /v1/esop/grants/{}/accruals` — no FR declares this in §3.
- `FR-ESOP-002` references `POST /v1/esop/vesting/run-batch` — no FR declares this in §3.
- `FR-ESOP-003` references `GET /v1/esop/valuations/{}` — no FR declares this in §3.
- `FR-ESOP-003` references `POST /v1/esop/valuations/{}/board-sign` — no FR declares this in §3.
- `FR-ESOP-003` references `POST /v1/esop/valuations/{}/dismiss` — no FR declares this in §3.
- `FR-ESOP-004` references `GET /v1/esop/members/{}/puts` — no FR declares this in §3.
- `FR-ESOP-004` references `GET /v1/esop/puts/{}` — no FR declares this in §3.
- `FR-ESOP-004` references `POST /v1/esop/puts/{}/approve` — no FR declares this in §3.
- `FR-ESOP-004` references `POST /v1/esop/puts/{}/reject` — no FR declares this in §3.
- `FR-ESOP-005` references `GET /v1/esop/leaver-outcomes/{}` — no FR declares this in §3.
- `FR-ESOP-005` references `POST /v1/esop/leaver-outcomes` — no FR declares this in §3.
- `FR-ESOP-005` references `POST /v1/esop/leaver-outcomes/{}/ceo-sign` — no FR declares this in §3.
- `FR-ESOP-005` references `POST /v1/esop/leaver-outcomes/{}/cfo-sign` — no FR declares this in §3.
- `FR-ESOP-006` references `GET /v1/esop/ma-events/{}` — no FR declares this in §3.
- `FR-ESOP-006` references `POST /v1/esop/ma-events/{}/accelerate` — no FR declares this in §3.
- `FR-ESOP-006` references `POST /v1/esop/ma-events/{}/board-sign` — no FR declares this in §3.
- `FR-HR-001` references `GET /v1/admin/members` — no FR declares this in §3.
- `FR-HR-001` references `GET /v1/admin/members/{}` — no FR declares this in §3.
- `FR-HR-001` references `PATCH /v1/admin/members` — no FR declares this in §3.
- `FR-HR-001` references `PATCH /v1/admin/members/{}` — no FR declares this in §3.
- `FR-HR-001` references `POST /v1/admin/members` — no FR declares this in §3.
- `FR-HR-001` references `POST /v1/admin/members/{}/transition` — no FR declares this in §3.
- `FR-HR-003` references `DELETE /v1/hr/members/{}/cccd-photo` — no FR declares this in §3.
- `FR-HR-003` references `GET /v1/hr/members/{}/cccd-photo` — no FR declares this in §3.
- `FR-HR-003` references `POST /v1/hr/members/{}/cccd-photo` — no FR declares this in §3.
- `FR-HR-003` references `POST /v1/hr/members/{}/cccd-photo/rotate` — no FR declares this in §3.
- `FR-HR-005` references `POST /v1/hr/policy-versions` — no FR declares this in §3.
- `FR-HR-005` references `PUT /v1/hr/tenant-policy-override` — no FR declares this in §3.
- `FR-HR-006` references `GET /v1/hr/members/{}/accrual-ledger` — no FR declares this in §3.
- `FR-HR-006` references `POST /v1/hr/accrual/run-batch` — no FR declares this in §3.
- `FR-HR-007` references `GET /v1/hr/onboarding/sagas/{}` — no FR declares this in §3.
- `FR-HR-007` references `POST /v1/hr/onboarding/start` — no FR declares this in §3.
- `FR-HR-007` references `POST /v1/hr/onboarding/{}/compensate` — no FR declares this in §3.
- `FR-HR-007` references `POST /v1/hr/onboarding/{}/retry` — no FR declares this in §3.
- `FR-HR-008` references `GET /v1/hr/members/{}/perf-history` — no FR declares this in §3.
- `FR-HR-008` references `POST /v1/hr/perf/snapshot` — no FR declares this in §3.
- `FR-INV-003` references `POST /v1/inv/stripe-secrets/rotate` — no FR declares this in §3.
- `FR-INV-003` references `POST /v1/inv/webhooks/stripe/{}` — no FR declares this in §3.
- `FR-INV-004` references `GET /v1/admin/unmatched-receipts` — no FR declares this in §3.
- `FR-INV-004` references `GET /v1/admin/wise-events` — no FR declares this in §3.
- `FR-INV-004` references `GET /v1/admin/wise-events/{}` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/admin/unmatched-receipts/{}/resolve` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/admin/wise-events/{}/restore` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/webhooks/wise/12345678` — no FR declares this in §3.
- `FR-INV-004` references `POST /v1/webhooks/wise/{}` — no FR declares this in §3.
- `FR-INV-005` references `POST /v1/inv/webhook-secrets/rotate` — no FR declares this in §3.
- `FR-INV-005` references `POST /v1/inv/webhooks/vietqr/acme-corp` — no FR declares this in §3.
- `FR-INV-005` references `POST /v1/inv/webhooks/vietqr/{}` — no FR declares this in §3.
- `FR-INV-006` references `GET /v1/inv/cash-app/allocations` — no FR declares this in §3.
- `FR-INV-006` references `GET /v1/inv/cash-app/unmatched` — no FR declares this in §3.
- `FR-INV-006` references `POST /v1/inv/cash-app/allocate-manual` — no FR declares this in §3.
- `FR-INV-006` references `POST /v1/inv/cash-app/dry-run` — no FR declares this in §3.
- `FR-INV-006` references `POST /v1/inv/cash-app/reverse` — no FR declares this in §3.
- `FR-KB-001` references `DELETE /v1/kb/documents` — no FR declares this in §3.
- `FR-KB-001` references `GET /v1/kb/documents` — no FR declares this in §3.
- `FR-KB-001` references `GET /v1/kb/documents/{}` — no FR declares this in §3.
- `FR-KB-001` references `PATCH /v1/kb/documents/{}` — no FR declares this in §3.
- `FR-KB-001` references `POST /v1/kb/documents` — no FR declares this in §3.
- `FR-KB-001` references `POST /v1/kb/documents/{}/archive` — no FR declares this in §3.
- `FR-KB-001` references `POST /v1/kb/documents/{}/versions` — no FR declares this in §3.
- `FR-KB-009` references `GET /v1/kb/docs/{}/translation-parity` — no FR declares this in §3.
- `FR-LEARN-001` references `GET /v1/learn/members/{}/mastery` — no FR declares this in §3.
- `FR-LEARN-001` references `GET /v1/learn/skills/tree` — no FR declares this in §3.
- `FR-LEARN-001` references `POST /v1/learn/skills` — no FR declares this in §3.
- `FR-LEARN-002` references `GET /v1/learn/members/{}/evidence` — no FR declares this in §3.
- `FR-LEARN-002` references `POST /v1/learn/evidence/{}/verify` — no FR declares this in §3.
- `FR-LEARN-002` references `POST /v1/learn/members/{}/evidence` — no FR declares this in §3.
- `FR-LEARN-003` references `GET /v1/learn/members/{}/vp` — no FR declares this in §3.
- `FR-LEARN-003` references `GET /v1/learn/vp/weights` — no FR declares this in §3.
- `FR-LEARN-003` references `POST /v1/learn/vp/rollup/trigger` — no FR declares this in §3.
- `FR-LEARN-003` references `POST /v1/learn/vp/weights` — no FR declares this in §3.
- `FR-LEARN-004` references `GET /v1/learn/councils/{}` — no FR declares this in §3.
- `FR-LEARN-004` references `POST /v1/learn/councils/{}/dismiss` — no FR declares this in §3.
- `FR-LEARN-004` references `POST /v1/learn/councils/{}/judges` — no FR declares this in §3.
- `FR-LEARN-006` references `GET /v1/learn/promotions/{}` — no FR declares this in §3.
- `FR-LEARN-006` references `POST /v1/learn/promotions/{}/ceo-sign` — no FR declares this in §3.
- `FR-LEARN-006` references `POST /v1/learn/promotions/{}/chro-sign` — no FR declares this in §3.
- `FR-LEARN-006` references `POST /v1/learn/promotions/{}/decline` — no FR declares this in §3.
- `FR-LEARN-007` references `GET /v1/learn/vp-rew/handoffs` — no FR declares this in §3.
- `FR-LEARN-007` references `POST /v1/learn/vp-rew/trigger` — no FR declares this in §3.
- `FR-MCP-001` references `POST /v1/mcp/register` — no FR declares this in §3.
- `FR-MCP-002` references `GET /v1/mcp/servers` — no FR declares this in §3.
- `FR-MCP-002` references `POST /v1/mcp/servers/deregister` — no FR declares this in §3.
- `FR-MCP-002` references `POST /v1/mcp/servers/heartbeat` — no FR declares this in §3.
- `FR-MCP-002` references `POST /v1/mcp/servers/register` — no FR declares this in §3.
- `FR-OBS-001` references `POST /v1/traces` — no FR declares this in §3.
- `FR-OKR-001` references `DELETE /v1/okr/objectives/{}/key_results/{}` — no FR declares this in §3.
- `FR-OKR-001` references `GET /v1/okr/cycles` — no FR declares this in §3.
- `FR-OKR-001` references `GET /v1/okr/objectives` — no FR declares this in §3.
- `FR-OKR-001` references `GET /v1/okr/teams` — no FR declares this in §3.
- `FR-OKR-001` references `PATCH /v1/okr/objectives/{}` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/cycles` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/cycles/{}/transition` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/key_results/{}/progress` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/objectives` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/objectives/{}/key_results` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/objectives/{}/transition` — no FR declares this in §3.
- `FR-OKR-001` references `POST /v1/okr/teams` — no FR declares this in §3.
- `FR-OKR-004` references `GET /v1/okr/auto-progress/runs` — no FR declares this in §3.
- `FR-OKR-004` references `GET /v1/okr/auto-progress/runs/{}` — no FR declares this in §3.
- `FR-OKR-004` references `POST /v1/okr/auto-progress/trigger` — no FR declares this in §3.
- `FR-OKR-005` references `GET /v1/okr/krs/{}/checkins` — no FR declares this in §3.
- `FR-OKR-005` references `GET /v1/okr/krs/{}/trend` — no FR declares this in §3.
- `FR-OKR-007` references `GET /v1/okr/retros` — no FR declares this in §3.
- `FR-OKR-007` references `POST /v1/okr/retros/{}/approve` — no FR declares this in §3.
- `FR-OKR-007` references `POST /v1/okr/retros/{}/dismiss` — no FR declares this in §3.
- `FR-OKR-007` references `POST /v1/okr/retros/{}/regenerate` — no FR declares this in §3.
- `FR-PROJ-001` references `DELETE /v1/proj/issues/{}` — no FR declares this in §3.
- `FR-PROJ-001` references `GET /v1/proj/issues` — no FR declares this in §3.
- `FR-PROJ-001` references `GET /v1/proj/issues/{}` — no FR declares this in §3.
- `FR-PROJ-001` references `PATCH /v1/proj/issues/issue-` — no FR declares this in §3.
- `FR-PROJ-001` references `PATCH /v1/proj/issues/{}` — no FR declares this in §3.
- `FR-PROJ-001` references `POST /v1/proj/issues` — no FR declares this in §3.
- `FR-PROJ-001` references `POST /v1/proj/issues/issue-1/links` — no FR declares this in §3.
- `FR-PROJ-001` references `POST /v1/proj/issues/{}/links` — no FR declares this in §3.
- `FR-PROJ-002` references `DELETE /v1/proj/decisions/<id>` — no FR declares this in §3.
- `FR-PROJ-002` references `GET /v1/brain/search` — no FR declares this in §3.
- `FR-PROJ-002` references `PATCH /v1/proj/decisions/<id>` — no FR declares this in §3.
- `FR-PROJ-002` references `PATCH /v1/proj/issues/issue-` — no FR declares this in §3.
- `FR-PROJ-002` references `POST /v1/proj/decisions/<id>/retract` — no FR declares this in §3.
- `FR-RES-001` references `GET /v1/res/matrix/runs/{}` — no FR declares this in §3.
- `FR-RES-001` references `GET /v1/res/members/{}/capacity` — no FR declares this in §3.
- `FR-RES-001` references `POST /v1/res/matrix/trigger` — no FR declares this in §3.
- `FR-RES-002` references `GET /v1/res/allocations/changes` — no FR declares this in §3.
- `FR-RES-002` references `POST /v1/res/allocations/{}/commit` — no FR declares this in §3.
- `FR-RES-003` references `GET /v1/res/flags/summary` — no FR declares this in §3.
- `FR-RES-003` references `GET /v1/res/weekly-digests` — no FR declares this in §3.
- `FR-RES-004` references `GET /v1/res/hiring-memos` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos/{}/ceo-sign` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos/{}/cfo-sign` — no FR declares this in §3.
- `FR-RES-004` references `POST /v1/res/hiring-memos/{}/dismiss` — no FR declares this in §3.
- `FR-RES-005` references `GET /v1/res/members/{}/ot-status` — no FR declares this in §3.
- `FR-REW-001` references `GET /v1/rew/comp/{}/decrypt` — no FR declares this in §3.
- `FR-REW-001` references `GET /v1/rew/members/{}/comp-history` — no FR declares this in §3.
- `FR-REW-002` references `GET /v1/rew/params/{}` — no FR declares this in §3.
- `FR-REW-002` references `POST /v1/rew/params` — no FR declares this in §3.
- `FR-REW-002` references `POST /v1/rew/replay-test/trigger` — no FR declares this in §3.
- `FR-REW-003` references `POST /v1/rew/p1-demotion-consents/{}/ceo-sign` — no FR declares this in §3.
- `FR-REW-003` references `POST /v1/rew/p1-demotion-consents/{}/cfo-sign` — no FR declares this in §3.
- `FR-REW-005` references `GET /v1/rew/payroll/runs/{}` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/cfo-sign` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/chro-sign` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/commit` — no FR declares this in §3.
- `FR-REW-005` references `POST /v1/rew/payroll/runs/{}/compute` — no FR declares this in §3.
- `FR-REW-006` references `POST /v1/rew/payslips/{}/verify` — no FR declares this in §3.
- `FR-REW-007` references `GET /v1/rew/members/{}/bp-balance` — no FR declares this in §3.
- `FR-REW-007` references `GET /v1/rew/members/{}/bp-ledger` — no FR declares this in §3.
- `FR-REW-007` references `POST /v1/rew/bp/debits` — no FR declares this in §3.
- `FR-REW-007` references `POST /v1/rew/bp/interest-accrual/trigger` — no FR declares this in §3.
- `FR-REW-008` references `GET /v1/rew/p3-distributions/{}` — no FR declares this in §3.
- `FR-REW-008` references `POST /v1/rew/p3-distributions/{}/ceo-sign` — no FR declares this in §3.
- `FR-REW-008` references `POST /v1/rew/p3-distributions/{}/cfo-sign` — no FR declares this in §3.
- `FR-REW-008` references `POST /v1/rew/p3-distributions/{}/execute` — no FR declares this in §3.
- `FR-REW-009` references `GET /v1/rew/payroll/batches/{}` — no FR declares this in §3.
- `FR-REW-009` references `GET /v1/rew/payroll/batches/{}/file` — no FR declares this in §3.
- `FR-REW-009` references `POST /v1/rew/payroll/batches/{}/confirm` — no FR declares this in §3.
- `FR-REW-009` references `POST /v1/rew/payroll/runs/{}/batch` — no FR declares this in §3.
- `FR-SKILL-201` references `GET /v1/skill/oci/bundles` — no FR declares this in §3.
- `FR-SKILL-201` references `POST /v1/skill/oci/pull` — no FR declares this in §3.
- `FR-SKILL-201` references `POST /v1/skill/oci/yank/{}` — no FR declares this in §3.
- `FR-TEN-001` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-TEN-002` references `DELETE /v1/admin/tenants/{}/plan/scheduled` — no FR declares this in §3.
- `FR-TEN-002` references `GET /v1/tenants/{}/plan` — no FR declares this in §3.
- `FR-TEN-002` references `GET /v1/tenants/{}/plan/history` — no FR declares this in §3.
- `FR-TEN-002` references `POST /v1/admin/founder/.../plan/override` — no FR declares this in §3.
- `FR-TEN-002` references `POST /v1/admin/founder/tenants/{}/plan/override` — no FR declares this in §3.
- `FR-TEN-002` references `POST /v1/admin/tenants/{}/plan` — no FR declares this in §3.
- `FR-TEN-003` references `DELETE /v1/subscriptions/{}` — no FR declares this in §3.
- `FR-TEN-003` references `GET /v1/charges/{}` — no FR declares this in §3.
- `FR-TEN-003` references `GET /v1/prices` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/customers` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/prices` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/refunds` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscription_items/{}/usage_records` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscription_schedules` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscriptions` — no FR declares this in §3.
- `FR-TEN-003` references `POST /v1/subscriptions/{}` — no FR declares this in §3.
- `FR-TEN-004` references `GET /v1/usage` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/documents/search` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/metering/internal/record` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/metering/period/close` — no FR declares this in §3.
- `FR-TEN-004` references `POST /v1/usage/correction` — no FR declares this in §3.
- `FR-TEN-101` references `DELETE /v1/subscriptions/{}` — no FR declares this in §3.
- `FR-TEN-101` references `POST /v1/admin/tenants` — no FR declares this in §3.
- `FR-TEN-101` references `POST /v1/setup_intents/{}/confirm` — no FR declares this in §3.
- `FR-TEN-101` references `POST /v1/signup/oidc-init` — no FR declares this in §3.
- `FR-TEN-102` references `POST /v1/inv/webhooks/momo/{}` — no FR declares this in §3.
- `FR-TEN-102` references `POST /v1/inv/webhooks/zalopay/{}` — no FR declares this in §3.
- `FR-TEN-103` references `GET /v1/account` — no FR declares this in §3.
- `FR-TEN-104` references `GET /v1/ten/offboarding/state/acme-corp` — no FR declares this in §3.
- `FR-TEN-104` references `GET /v1/ten/offboarding/state/{}` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/cancel` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/extend` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/finalize-termination` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/force-advance` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/initiate` — no FR declares this in §3.
- `FR-TEN-104` references `POST /v1/ten/offboarding/restore-from-dead-letter` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/ceo-sign` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/challenge` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/clo-sign` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/cso-sign` — no FR declares this in §3.
- `FR-TEN-202` references `POST /v1/ten/hostile-overrides/{}/execute` — no FR declares this in §3.
- `FR-TIME-001` references `GET /v1/time/entries` — no FR declares this in §3.
- `FR-TIME-001` references `GET /v1/time/entries/{}` — no FR declares this in §3.
- `FR-TIME-001` references `POST /v1/time/entries` — no FR declares this in §3.
- `FR-TIME-001` references `POST /v1/time/entries/{}/correct` — no FR declares this in §3.

---

**Interpretation**: Orphans are normal for (a) future-FR placeholders, (b) external APIs (Stripe, ACRA, GDT), (c) internal-only endpoints not surfaced in §3. Review individually before flagging.
---

# §2 — Implementation order (topological)


_Generated 2026-05-17 — 241 FRs in 13 dependency layers._

Each **layer** can be built in parallel (no cross-dependencies inside a layer). Layers MUST be built in order.

Within a layer, FRs are sorted alphabetically — pick by module ownership.

## Layer 0 (9 FRs — buildable in parallel)

- **FR-AI-003** [MUST, 5h, slice 1] — BRAIN audit-row bridge — canonical Writer for AI Gateway
- **FR-AI-005** [MUST, 5h, slice 1] — Tenant-policy YAML loader — per-tenant cap + warn + override + residency
- **FR-AI-007** [MUST, 4h, slice 2] — Provider cost-table loader — YAML-backed, hot-reloadable rate table
- **FR-AI-019** [SHOULD, 12h, slice 4] — Self-hosted BGE-M3 embeddings (single L4 GPU sidecar) + ONNX-CPU fallback + adap
- **FR-AUTH-001** [MUST, 8h, slice 1] — Tenant create — root-admin in tenant 0 calls POST /v1/admin/tenants with idempot
- **FR-CHAT-001** [MUST, 8h, slice 1] — Mattermost v9.x fork at pinned MIT-Apache commit + automated license-drift watch
- **FR-DOCS-001** [SHOULD, 14h, slice 1] — Server-render NFR catalog + Risk Register + FR catalog at build time — Pagefind-
- **FR-EMAIL-001** [MUST, 12h, slice 1] — EMAIL Stalwart Rust mail server deployment — JMAP + IMAP + SMTP + ManageSieve + 
- **FR-OBS-001** [MUST, 10h, slice 1] — OTel Collector + LGTM stack (Loki + Prometheus + Tempo + Grafana) with mTLS ingr

## Layer 1 (12 FRs — buildable in parallel)

- **FR-AI-001** [MUST, 8h, slice 1] — AI Gateway cost-ledger pre-call check
- **FR-AI-014** [MUST, 8h, slice 3] — Persona-version system-prompt injection from BRAIN memories/personas/<handle>.md
- **FR-AI-020** [COULD, 8h, slice 4] — BGE-reranker-v2-m3 cross-encoder for KB reranking (per-region sidecar; CPU fallb
- **FR-AUTH-002** [MUST, 6h, slice 1] — Subject create — POST /v1/admin/subjects with bcrypt + role allow-list + idempot
- **FR-AUTH-003** [MUST, 12h, slice 1] — RLS enforcement at every tenant-scoped table — USING + WITH CHECK + per-connecti
- **FR-EMAIL-004** [MUST, 6h, slice 1] — EMAIL DKIM signing + ARC chain forward + BIMI brand indicator — RFC 6376 + RFC 8
- **FR-EMAIL-005** [MUST, 12h, slice 2] — EMAIL CaMeL dual-LLM security layer — Privileged-LLM plans, Quarantined-LLM pars
- **FR-EMAIL-011** [MUST, 5h, slice 2] — EMAIL DSAR message export — every message a subject authored or received + chain
- **FR-OBS-003** [MUST, 8h, slice 1] — Per-service RED metrics (rate/errors/duration) via cyberos-obs-sdk shared crate 
- **FR-OBS-006** [SHOULD, 6h, slice 2] — Tail-based sampling at OTel collector — 100% errors/5xx/slow/flagged + 10% norma
- **FR-SKILL-101** [MUST, 6h, slice 1] — Skill BRAIN integration — skill.invoked_started + skill.invoked_completed audit 
- **FR-TEN-001** [MUST, 5h, slice 1] — TEN tenant provisioning CLI — `cyberos-ten provision` ops-driven flow with schem

## Layer 2 (11 FRs — buildable in parallel)

- **FR-AI-002** [MUST, 6h, slice 1] — AI Gateway cost-ledger post-call reconcile
- **FR-AI-004** [MUST, 3h, slice 1] — Cost-hold expiry cleanup job — refund unsettled holds + emit audit
- **FR-AUTH-004** [MUST, 12h, slice 1] — JWT issuance + JWKS endpoint (RS256) with tenant_id + agent_persona + scope_gran
- **FR-AUTH-102** [MUST, 10h, slice 1] — AUTH TOTP (RFC 6238) + WebAuthn Level 3 MFA — closed factor enum + enrolment FSM
- **FR-BRAIN-101** [MUST, 18h, slice 1] — Layer-2 ingest pipeline (binlog → pgvector + Apache AGE) — chain-anchor verifica
- **FR-EMAIL-009** [MUST, 4h, slice 1] — EMAIL outbound 1:1 send — DKIM-signed via FR-EMAIL-004 + AM confirm-before-send 
- **FR-PROJ-001** [MUST, 12h, slice 1] — PROJ Issue + Cycle + Engagement schema — RLS + cross-module linkable + status FS
- **FR-SKILL-102** [MUST, 10h, slice 1] — Self-hosted OCI registry for .skill bundles — cosign signing + tenant-scoped + i
- **FR-SKILL-103** [MUST, 7h, slice 1] — SKILL.md frontmatter extension — allowed_brain_scopes + allowed_tools + version 
- **FR-TEN-002** [MUST, 4h, slice 1] — 3 plan tiers (Starter / Team / Enterprise) hardcoded with per-tier caps
- **FR-TEN-104** [MUST, 12h, slice 1] — TEN 90-day offboarding contract — closed 4-state FSM (Active → Terminating-A → T

## Layer 3 (22 FRs — buildable in parallel)

- **FR-AI-006** [MUST, 6h, slice 2] — Model-alias resolution (chat.smart → bedrock:claude-3.5-sonnet) with per-tenant 
- **FR-AUTH-005** [MUST, 8h, slice 1] — Admin REST: list tenants + list subjects + revoke subject + unrevoke + cursor pa
- **FR-AUTH-006** [MUST, 6h, slice 1] — cyberos-auth bootstrap CLI: tenant 0 + root-admin + initial signing key + sweepe
- **FR-AUTH-103** [MUST, 12h, slice 1] — AUTH SAML 2.0 SSO — SP-initiated flow + per-tenant IdP config + XML signature ve
- **FR-AUTH-105** [MUST, 8h, slice 1] — AUTH Passkey enrolment + login — discoverable credentials (resident keys) + auto
- **FR-AUTH-106** [SHOULD, 8h, slice 1] — Impossible-travel detection + adaptive MFA challenge
- **FR-BRAIN-102** [MUST, 10h, slice 1] — Layer-2 rebuild-from-Layer-1 CI gate — deterministic rebuild + spot-check + 30mi
- **FR-BRAIN-103** [MUST, 18h, slice 1] — brain-sync daemon — laptop A ↔ Cloud BRAIN ↔ laptop B with sync_class gating + C
- **FR-CHAT-002** [MUST, 10h, slice 1] — cyberos-chat-authbridge plugin — Mattermost auth delegates to FR-AUTH-004 JWT wi
- **FR-EMAIL-002** [MUST, 6h, slice 1] — EMAIL Stalwart authbridge plugin — JMAP/IMAP/SMTP auth delegates to AUTH JWT val
- **FR-EMAIL-003** [MUST, 16h, slice 2] — EMAIL Missive-style team UX — shared inbox, thread assignment, internal comments
- **FR-EMAIL-007** [SHOULD, 6h, slice 1] — EMAIL convert-to-issue — one-click create FR-PROJ issue from message with thread
- **FR-EMAIL-010** [MUST, 5h, slice 1] — EMAIL bulk send (≥ 10 recipients) — AM + CFO/marketing dual-approval token + sup
- **FR-MCP-001** [MUST, 12h, slice 4] — MCP Gateway 2025-11-25 spec compliance — initialize + tools/list + tools/call + 
- **FR-OBS-002** [MUST, 12h, slice 1] — Tenant-aware Grafana proxy (Rust) — AST-injects tenant_id into PromQL/LogQL/Trac
- **FR-PROJ-002** [MUST, 7h, slice 1] — BRAIN-anchored proj.decision row per Issue state change — reason + prior_chain l
- **FR-PROJ-005** [MUST, 4h, slice 2] — Rate-card schema per Engagement — (role × currency × hourly_rate × billable_defa
- **FR-PROJ-009** [MUST, 5h, slice 2] — BRAIN_LINK schema — Issue ↔ BRAIN memory linkage (cites | implements | supersede
- **FR-SKILL-104** [MUST, 12h, slice 1] — Capability broker — subprocess sandbox enforces allowed_tools + allowed_brain_sc
- **FR-SKILL-201** [MUST, 8h, slice 1] — SKILL OCI registry deploy for `.skill` bundles — R3 distribution stage with sign
- **FR-TEN-105** [MUST, 8h, slice 2] — TEN signed-bundle export — deterministic zip + Ed25519 signature + BRAIN audit a
- **FR-TEN-202** [SHOULD, 5h, slice 1] — TEN hostile-termination override — legal-trigger fast-track with CEO+CLO+CSO tri

## Layer 4 (26 FRs — buildable in parallel)

- **FR-AI-008** [MUST, 10h, slice 2] — LiteLLM-derived multi-provider router with retry + 30s failover SLA
- **FR-AI-015** [MUST, 6h, slice 3] — ZDR (Zero Data Retention) attestation table + enforcement when tenant policy req
- **FR-AI-016** [MUST, 8h, slice 4] — Tenant residency pinning (sg-1 / eu-1 / us-1 / vn-1) propagating to provider reg
- **FR-AUTH-101** [MUST, 12h, slice 1] — AUTH 22-role RBAC catalogue — closed enum + permission matrix + role-assignment 
- **FR-AUTH-107** [SHOULD, 4h, slice 1] — HIBP password breach check (k-anonymity) on signup + rotation
- **FR-BRAIN-104** [SHOULD, 28h, slice 2] — Tauri 2.x desktop app — macOS + Windows + Linux signed/notarised + auto-update +
- **FR-BRAIN-106** [MUST, 6h, slice 1] — BRAIN sync_class enforcement — private vs shareable + ACL filtering + structural
- **FR-CHAT-003** [MUST, 6h, slice 1] — Per-tenant CHAT deployment — AWS Fargate + RDS Multi-AZ + Redis ElastiCache with
- **FR-DOC-006** [MUST, 8h, slice 2] — DOC identity verification — 4 methods (WebAuthn / VNeID / SMS-OTP / email-link) 
- **FR-MCP-002** [MUST, 6h, slice 2] — MCP per-module server registration + heartbeat lifecycle — 3-miss → unhealthy wi
- **FR-MCP-003** [MUST, 3h, slice 2] — MCP SEP-986 naming convention validator — `cyberos.{module}.{verb}_{noun}` patte
- **FR-MCP-004** [MUST, 10h, slice 2] — OAuth 2.1 PKCE authorization-code flow with audience-bound tokens for MCP server
- **FR-OBS-007** [MUST, 10h, slice 3] — obs-router: Alertmanager → CUO obs.triage-alert@1 skill → CHAT (≥0.70 conf) OR P
- **FR-OBS-008** [MUST, 14h, slice 3] — obs-compliance-view: pre-built read-only views (EU AI Act / PDPL / SOC 2 / ISO 2
- **FR-PROJ-003** [MUST, 10h, slice 2] — Yjs CRDT for issue description + comment-body fields; LWW for scalar metadata; r
- **FR-PROJ-004** [MUST, 5h, slice 2] — Issue lifecycle FSM — backlog → todo → in-progress → in-review → done | cancelle
- **FR-PROJ-006** [MUST, 6h, slice 2] — Billable cascade — Member-override → task-class → role-default → fallback; resol
- **FR-PROJ-010** [SHOULD, 4h, slice 3] — Citation drift detector — nightly sweep flags stale BRAIN_LINKs (deleted target,
- **FR-PROJ-013** [MUST, 6h, slice 3] — Estimate calibration snapshot — per-member per-task-class nightly batch with Bay
- **FR-PROJ-014** [MUST, 10h, slice 3] — Kanban Board view — drag/drop status transition + keyboard-first navigation + 60
- **FR-PROJ-015** [MUST, 8h, slice 3] — Timeline view — cycle window × assignee swimlane with day-grid layout, drag-resi
- **FR-PROJ-016** [SHOULD, 10h, slice 3] — Gantt view with dependency arrows — issue-to-issue precedence + critical path hi
- **FR-SKILL-105** [MUST, 9h, slice 2] — brain-capture@1 skill bundle — canonical SDK-style entry point for emitting BRAI
- **FR-SKILL-108** [MUST, 7h, slice 3] — vn-mst-validate@1 skill — Vietnamese Tax ID (MST) validation against General Dep
- **FR-TEN-106** [MUST, 5h, slice 2] — TEN permanent-delete attestation — CSO + CLO dual-sign + chain-anchored evidence
- **FR-TIME-004** [SHOULD, 6h, slice 2] — TIME auto-detect proposals — Member-confirm suggestions from PROJ activity (stat

## Layer 5 (35 FRs — buildable in parallel)

- **FR-AI-009** [MUST, 6h, slice 2] — Circuit breaker per (provider, model) with half-open recovery probing
- **FR-AI-010** [SHOULD, 8h, slice 2] — Streaming SSE end-to-end (token-by-token to client)
- **FR-AI-011** [MUST, 6h, slice 3] — Presidio EN-base PII redaction in-flight (every prompt)
- **FR-AI-017** [SHOULD, 8h, slice 4] — Per-tenant Redis response cache keyed by (tenant × redacted-prompt × model × per
- **FR-AI-022** [MUST, 8h, slice 5] — OpenTelemetry trace + span emission for every call (caller → router → provider →
- **FR-AI-104** [SHOULD, 12h, slice 1] — AI VN provider integration — Viettel Cloud + FPT Cloud as Vn1-residency LLM/embe
- **FR-AUTH-104** [MUST, 10h, slice 1] — AUTH OIDC SSO — RFC 8414 discovery + RFC 7517 JWKS rotation + per-tenant IdP con
- **FR-AUTH-108** [MUST, 6h, slice 1] — AUTH Lumi tenant-identity JWT shape — agent_persona + tenant_residency + lumi_or
- **FR-AUTH-109** [MUST, 5h, slice 1] — AUTH stub → full migration enforcer — 30-day grace window + cutover timestamp + 
- **FR-BRAIN-105** [MUST, 7h, slice 2] — cyberos doctor — watched-folders integrity invariants (manifest ↔ filesystem ↔ H
- **FR-CHAT-004** [MUST, 12h, slice 1] — PGroonga + custom Vietnamese bigram tokeniser — VN message search with ≥ 80% rec
- **FR-CHAT-005** [MUST, 10h, slice 1] — BRAIN bridge — Postgres logical replication from chat to BRAIN Layer-3 ingest wi
- **FR-CHAT-011** [MUST, 6h, slice 2] — Mobile push delivery — APNS + FCM with privacy-preserving payload (title + sende
- **FR-CRM-001** [MUST, 6h, slice 1] — CRM Account/Contact/Deal Postgres schema — closed entity primitives + custom pip
- **FR-CUO-101** [MUST, 12h, slice 2] — CUO Phase 2 — LangGraph supervisor + LiteLLM cascade + confidence-band escalatio
- **FR-DOC-001** [MUST, 8h, slice 1] — DOC Document repository — S3 Object-Lock Compliance bucket + per-tenant residenc
- **FR-HR-001** [MUST, 6h, slice 1] — HR Member schema — profile + role + level + contract type + leave balance + sabb
- **FR-INV-003** [MUST, 8h, slice 2] — INV Stripe webhook handler — Stripe-Signature verify + closed event-type allowli
- **FR-INV-004** [SHOULD, 6h, slice 1] — Wise webhook handler for multi-currency receipts (USD / EUR / GBP / SGD / JPY)
- **FR-INV-005** [MUST, 6h, slice 2] — INV VietQR / Napas247 webhook handler — HMAC-SHA256 signature + idempotent recei
- **FR-KB-001** [MUST, 6h, slice 1] — KB Document schema — slug + markdown body + YAML frontmatter + closed category e
- **FR-MCP-005** [MUST, 3h, slice 2] — MCP Protected Resource Metadata (RFC 9728) at `/.well-known/oauth-protected-reso
- **FR-MCP-006** [MUST, 6h, slice 2] — MCP tool-annotation gating — destructive / write / external-effect tools require
- **FR-MCP-007** [MUST, 10h, slice 3] — MCP Tasks primitive — long-running tool calls with status polling + resume-on-re
- **FR-MCP-008** [MUST, 6h, slice 3] — MCP Elicitation — server-initiated structured prompts for mid-call user input (c
- **FR-OBS-009** [MUST, 8h, slice 3] — Chain-of-custody manifest with Ed25519 signature on every compliance export — PD
- **FR-OKR-001** [MUST, 6h, slice 1] — OKR Objective × Key Result schema — Company → Team → Member cascade + quarterly 
- **FR-PROJ-007** [MUST, 6h, slice 2] — Three billing modes — Time & Materials, Fixed-Fee, Retainer — with mode-aware ro
- **FR-PROJ-008** [MUST, 5h, slice 2] — BRAIN audit row per issue mutation — chained to PROJ history_event table with fi
- **FR-PROJ-017** [MUST, 8h, slice 3] — Brief Modal — issue deep-view with Yjs description editor + threaded comments + 
- **FR-PROJ-018** [MUST, 8h, slice 3] — Liquid-Glass design tokens (tokens.proj.css) + axe-core CI accessibility gate + 
- **FR-SKILL-106** [SHOULD, 4h, slice 3] — brain-sync@1 skill bundle — operator-facing sync trigger that defers to Stage 4 
- **FR-SKILL-109** [MUST, 7h, slice 3] — vn-bank-transfer@1 skill — VietQR + Napas247 transfer-code generator with bank-p
- **FR-TEN-103** [MUST, 10h, slice 2] — 4-residency provisioning — sg-1 / eu-1 / us-1 / vn-1 region pinning across Postg
- **FR-TIME-001** [MUST, 5h, slice 1] — TIME TimeEntry append-only schema — correction_to link semantics + tenant-scoped

## Layer 6 (58 FRs — buildable in parallel)

- **FR-AI-012** [MUST, 10h, slice 3] — VN-PII Presidio plugin (CCCD · MST · VN phone · NĐD · VN address · bank account)
- **FR-AI-018** [MUST, 6h, slice 4] — Cross-tenant cache leak property-test (hard zero) — 200K random ops + 7 regressi
- **FR-AI-021** [MUST, 14h, slice 5] — cyberos-ai operator CLI (usage · models · policy · failover · invoice · breaker 
- **FR-BRAIN-107** [MUST, 14h, slice 2] — BRAIN capture daemon — Rust + notify crate FS watcher with rate-limit + content-
- **FR-CHAT-006** [MUST, 12h, slice 2] — Slack import — `cyberos-chat import slack` with 8-step idempotent checkpoint-dri
- **FR-CHAT-008** [MUST, 6h, slice 2] — @lumi mention parser — message mentions trigger CUO routing + BRAIN capture row 
- **FR-CHAT-012** [MUST, 6h, slice 2] — DSAR export — Data Subject Access Request: every message a subject authored + ch
- **FR-CRM-003** [MUST, 4h, slice 5] — CRM VN account types + MST — legal entity classification (Sole/LLC/JSC/FDI) + ta
- **FR-CRM-004** [MUST, 6h, slice 5] — CRM convert-to-engagement — deal.won → PROJ Engagement creation with rate card +
- **FR-CRM-005** [MUST, 6h, slice 6] — CRM CUO crm.next-action@1 skill — AI-ranked top-3 next moves per open deal with 
- **FR-CRM-006** [SHOULD, 5h, slice 6] — CRM AI lead scoring — contact-creation-time score + nightly refresh based on act
- **FR-CRM-007** [SHOULD, 5h, slice 6] — CRM win/loss analysis CUO draft — auto-generate analysis at deal close + BRAIN m
- **FR-CRM-009** [MUST, 4h, slice 7] — CRM vn-bank-transfer skill — VietQR payment image generation for deal collection
- **FR-CUO-102** [MUST, 5h, slice 6] — CUO Postgres checkpointer for LangGraph state — persists supervisor graph state 
- **FR-CUO-104** [MUST, 10h, slice 6] — CUO topological walk of `depends_on` chain — orchestrates multi-step skill invoc
- **FR-DOC-002** [MUST, 16h, slice 3] — DOC eIDAS QTSP integration — GlobalSign or Cryptomathic partner for EU residency
- **FR-DOC-003** [MUST, 12h, slice 3] — DOC AATL CA integration — Adobe Approved Trust List CA partner (DigiCert / Entru
- **FR-DOC-004** [MUST, 16h, slice 3] — DOC VN CA chain — VNeID + VnPay/MK Group/Viettel-CA partners for VN-residency qu
- **FR-DOC-005** [MUST, 10h, slice 2] — DOC multi-party signing workflow — ordered + parallel + counter-sign with remind
- **FR-DOC-007** [MUST, 5h, slice 1] — DOC lifecycle metadata — parties + effective_date + expiry_date + renewal_terms 
- **FR-DOC-010** [SHOULD, 10h, slice 3] — DOC third-party import — DocuSign / Adobe Sign / HelloSign migration with LTV (l
- **FR-EMAIL-006** [SHOULD, 5h, slice 1] — EMAIL tracked-domain → CRM auto-link — inbound message from tenant-tracked domai
- **FR-ESOP-001** [MUST, 5h, slice 1] — ESOP SP grant schema — Stock Plan grant with 4-year vesting + 12-month cliff def
- **FR-HR-002** [MUST, 4h, slice 6] — HR 5 contract types — indefinite + fixed_term + probation + part_time + contract
- **FR-HR-003** [MUST, 5h, slice 6] — HR CCCD photo KMS — separate keyspace for VN citizen ID photos with sev-1 access
- **FR-HR-004** [MUST, 5h, slice 6] — HR 8 leave types — annual/sick/maternity/paternity/sabbatical/unpaid/bereavement
- **FR-HR-005** [MUST, 4h, slice 6] — HR Decree 145/2020 working-hour caps + Decree 152/2020 SI rates — version-pinned
- **FR-HR-007** [MUST, 10h, slice 6] — HR onboarding saga — orchestrates AUTH + TIME + LEARN + KB + CHAT + REW provisio
- **FR-HR-008** [MUST, 6h, slice 7] — HR performance signal aggregator — read-only consumer of PROJ + TIME + LEARN sig
- **FR-HR-009** [MUST, 8h, slice 7] — HR termination workflow — Good-Leaver / Bad-Leaver branch with CFO+CEO co-sign +
- **FR-INV-006** [MUST, 8h, slice 2] — INV cash application — closed 4-step matching cascade (exact-ref → amount+date →
- **FR-KB-002** [MUST, 5h, slice 4] — KB server-side renderer — markdown → sanitised HTML (ammonia) + sanitised plaint
- **FR-KB-003** [MUST, 5h, slice 4] — KB 3 permission tiers — public / org-only / role-restricted with share-link toke
- **FR-KB-005** [MUST, 6h, slice 5] — KB BGE-M3 semantic search — BRAIN Layer 2 vector ingest + dense embedding query 
- **FR-KB-008** [MUST, 5h, slice 5] — KB runbook category — applicability tags (provider / region / severity) for OBS 
- **FR-KB-009** [SHOULD, 4h, slice 5] — KB dual-language `translation_of` link — vi/en pairing with locale-aware reader 
- **FR-LEARN-001** [MUST, 6h, slice 7] — LEARN skill tree schema — 1-5 mastery levels per skill per Member with parent-ch
- **FR-LEARN-003** [MUST, 6h, slice 7] — LEARN VP (Voting Power) deterministic nightly roll-up — aggregates PROJ + TIME +
- **FR-OBS-004** [MUST, 6h, slice 2] — LangSmith integration for AI traces — self-hosted + per-tenant opt-in + redacted
- **FR-OKR-002** [MUST, 4h, slice 3] — OKR 3 KR types — hit_target + improvement + milestone with type-specific progres
- **FR-OKR-003** [MUST, 10h, slice 3] — OKR KR progress_source DSL — declarative query against PROJ / INV / HR / LEARN m
- **FR-OKR-005** [MUST, 5h, slice 3] — OKR weekly check-in — 1-10 confidence + rationale per KR with rolling 4-week his
- **FR-OKR-007** [SHOULD, 6h, slice 3] — OKR quarterly retro CUO draft — auto-generated retro with face-saving Vietnamese
- **FR-PORTAL-003** [MUST, 10h, slice 1] — PORTAL external IdP — SAML 2.0 + OIDC sign-in for client-tenant users + SCIM 2.0
- **FR-PORTAL-006** [MUST, 6h, slice 2] — PORTAL client-initiated workflows — new project request / billing inquiry / supp
- **FR-PROJ-011** [MUST, 6h, slice 3] — Blocker detector from comment stream — `blocked by` parser + dwell-time monitor 
- **FR-PROJ-012** [MUST, 8h, slice 3] — Cycle-review draft generator — CUO/COO-persona LLM compose at cycle close with c
- **FR-RES-001** [MUST, 10h, slice 7] — RES capacity-vs-demand matrix — nightly join across HR + PROJ + TIME + LEARN pro
- **FR-RES-004** [MUST, 8h, slice 8] — RES hiring memo CUO draft — skill-gap × CRM pipeline trigger → CEO+CFO review qu
- **FR-REW-001** [MUST, 6h, slice 1] — REW 3P income schema — P1 Base + P2 Allowance + P3 Performance with separate enc
- **FR-REW-009** [MUST, 5h, slice 2] — REW VietQR bank payroll batch send — bulk transfer file generation with CFO manu
- **FR-SKILL-107** [COULD, 3h, slice 1] — synthesis-author@1 skill — nightly multi-brain auto-evolve composes derived memo
- **FR-SKILL-110** [MUST, 11h, slice 3] — vn-vat-invoice@1 skill — Vietnamese e-invoice (hóa đơn) Decree 123 XML emitter w
- **FR-TIME-002** [MUST, 5h, slice 1] — TIME timer start/stop — single-active-timer per Member + auto-stop on logout + ≤
- **FR-TIME-003** [MUST, 6h, slice 1] — TIME manual entry form — retroactive time logging with date validation + per-day
- **FR-TIME-005** [MUST, 5h, slice 1] — TIME billable flag cascade — 4-step resolver (entry override → project default →
- **FR-TIME-006** [MUST, 6h, slice 1] — TIME weekly approval flow — Member submit → AM (engagement_admin) review → CFO v
- **FR-TIME-007** [MUST, 4h, slice 1] — TIME VN Labour Code Art. 107 OT cap — hard-block at entry write when monthly OT 

## Layer 7 (41 FRs — buildable in parallel)

- **FR-AI-013** [MUST, 8h, slice 3] — VN-PII recall ≥ 99% per-recognizer CI gate on 200-sample fixture
- **FR-BRAIN-108** [MUST, 12h, slice 2] — BRAIN search — vector + graph + full-text in parallel + RRF fusion + BGE-rerank 
- **FR-BRAIN-109** [MUST, 8h, slice 2] — Claude Code hook capture — UserPromptSubmit + PostToolUse + Stop hooks emit BRAI
- **FR-BRAIN-111** [MUST, 9h, slice 2] — BRAIN pre-ingest PII detection — Presidio EN + custom VN recognisers; ≥ 99.5% he
- **FR-CHAT-007** [SHOULD, 8h, slice 2] — Zalo manual export importer — `cyberos-chat import zalo --bundle.zip` with VN-Un
- **FR-CHAT-009** [SHOULD, 6h, slice 2] — Retro-capture flow — `@lumi remember the last N messages` with per-message opt-i
- **FR-CRM-002** [MUST, 8h, slice 5] — CRM activity feed — auto-log inbound email + outbound send + chat mention + cale
- **FR-CRM-008** [MUST, 3h, slice 7] — CRM vn-mst-validate skill — synchronous GDT lookup on Account write to confirm M
- **FR-CUO-103** [MUST, 4h, slice 6] — CUO Phase 2 trace rows include prompt + model + temperature + seed for determini
- **FR-CUO-105** [MUST, 6h, slice 6] — CUO per-step rollback on chain failure — execute compensating actions in reverse
- **FR-DOC-008** [MUST, 4h, slice 1] — DOC expiry alert cascade — 90/30/7-day notifications to parties + CLO with dedup
- **FR-DOC-009** [SHOULD, 6h, slice 1] — DOC renewal proposal CUO draft — auto-generate renewal terms + price adjustment 
- **FR-DOC-011** [MUST, 8h, slice 3] — DOC PAdES-B-LT format + year-9 LTV re-stamping — extend B-T signatures with vali
- **FR-ESOP-002** [MUST, 4h, slice 1] — ESOP monthly vesting accrual deterministic batch — runs EOM tenant_tz computing 
- **FR-ESOP-003** [MUST, 5h, slice 1] — ESOP annual valuation — CFO base + Board multiplier sign-off with immutable shar
- **FR-ESOP-005** [MUST, 5h, slice 2] — ESOP Good/Bad Leaver branch on HR offboarding — CFO+CEO co-sign to apply forfeit
- **FR-ESOP-006** [SHOULD, 5h, slice 2] — ESOP M&A acceleration trigger — Board declares M&A event + 5-business-day Member
- **FR-ESOP-007** [SHOULD, 6h, slice 2] — ESOP Member dashboard — personal view only (own grants + vesting + estimated val
- **FR-HR-006** [MUST, 4h, slice 6] — HR annual leave accrual nightly batch — Decree 145 formula (1d/month + 1d/5yr se
- **FR-KB-004** [MUST, 6h, slice 5] — KB FTS5 + PGroonga lexical search — VN bigram tokenisation + English stemming + 
- **FR-KB-006** [MUST, 4h, slice 5] — KB BGE-rerank-v2-m3 cross-encoder — reranks top-K results from FR-KB-004 lexical
- **FR-LEARN-002** [MUST, 4h, slice 7] — LEARN bằng cấp + chứng chỉ — degree + certification evidence types with issuer +
- **FR-LEARN-004** [MUST, 10h, slice 7] — LEARN Hội đồng Chuyên môn (Specialist Council) — 3-5 judges + multi-dim scoring 
- **FR-LEARN-007** [MUST, 4h, slice 7] — LEARN VP score → REW BP fund distribution handoff — quarter-close trigger emits 
- **FR-OBS-005** [MUST, 8h, slice 2] — W3C TraceContext correlation across logs/metrics/traces/AI-traces — propagate, e
- **FR-OKR-004** [MUST, 5h, slice 3] — OKR auto-progress nightly batch — resolves all KR progress_sources + updates cur
- **FR-OKR-006** [MUST, 6h, slice 3] — OKR Monday-morning CUO digest — auto-progress + check-ins → founder summary deli
- **FR-PORTAL-004** [MUST, 8h, slice 2] — PORTAL SCIM deprovision — session invalidation ≤ 30 s on IdP user removal + grac
- **FR-PORTAL-005** [SHOULD, 6h, slice 2] — PORTAL branded Genie chat — CUO scope-narrowed by JWT scope_grants + per-Engagem
- **FR-RES-002** [MUST, 12h, slice 8] — RES allocation Gantt UI — drag-rebalance interface over capacity matrix with opt
- **FR-RES-003** [MUST, 4h, slice 8] — RES over/under-allocation flags — 110% warning / 60% under-utilization threshold
- **FR-RES-005** [MUST, 4h, slice 8] — RES VN Labour Code Art. 107 OT cap hard-block — propose-time validation gate pre
- **FR-REW-002** [MUST, 6h, slice 1] — REW parameter versioning — immutable versioned formula parameters with 100% repl
- **FR-REW-003** [MUST, 4h, slice 1] — REW P1 protection invariant — DB CHECK constraint + service-layer guard forbiddi
- **FR-REW-004** [MUST, 6h, slice 1] — REW statutory deductions — BHXH 10.5% + BHYT 1.5% + BHTN 1% + PIT progressive pe
- **FR-REW-005** [MUST, 8h, slice 2] — REW monthly payroll compute + CFO+CHRO co-sign commit gate — orchestrates 3P + d
- **FR-REW-007** [MUST, 5h, slice 2] — REW BP (Bonus Points) ledger with ACB-rate interest accrual nightly + per-Member
- **FR-REW-010** [MUST, 3h, slice 1] — REW BRAIN structural exclusion CI gate — no comp fields appear in BRAIN-ingest p
- **FR-TEN-005** [MUST, 5h, slice 2] — TEN vertical-pack pricing add-on — per-pack monthly fee (not per-seat) on top of
- **FR-TEN-201** [MUST, 16h, slice 1] — TEN Singapore HoldCo flip CLI — `cyberos-ten holdco-flip` orchestrates ACRA fili
- **FR-TIME-009** [MUST, 6h, slice 1] — TIME per-cycle billable rollup → INV — per-Member × role × Engagement aggregatio

## Layer 8 (11 FRs — buildable in parallel)

- **FR-BRAIN-110** [MUST, 6h, slice 2] — BRAIN capture daemon supervision — systemd + launchd units + /healthz + watchdog
- **FR-CHAT-010** [MUST, 5h, slice 2] — Decommission signal — (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95 over 14-da
- **FR-EMAIL-008** [SHOULD, 8h, slice 2] — EMAIL Genie prefix — inbound subject prefix routes message to Genie (Branded AI)
- **FR-ESOP-004** [MUST, 8h, slice 2] — ESOP put-option exec flow — Year 3+ eligibility + per-Member annual cap + CFO ap
- **FR-INV-001** [MUST, 8h, slice 1] — INV invoice substrate — draft invoices from TIME per-cycle rollup with rate-card
- **FR-KB-007** [MUST, 8h, slice 5] — KB Ask-this-page Q&A — CUO-grounded answer over current + linked docs with span-
- **FR-LEARN-005** [MUST, 5h, slice 7] — LEARN per-judge score isolation — never exit LEARN boundary; HR receives only su
- **FR-LEARN-006** [MUST, 5h, slice 7] — LEARN promotion approval workflow — CEO + CHRO sign-off after council vote with 
- **FR-REW-006** [MUST, 6h, slice 2] — REW byte-identical payslip PDF render — Tectonic + pinned fonts produces determi
- **FR-REW-008** [MUST, 6h, slice 2] — REW quarterly P3 distribution from BP fund — CEO+CFO sign-off + LEARN-007 VP sha
- **FR-TEN-004** [MUST, 8h, slice 1] — 4-axis metering — seats · API · AI tokens · storage (BRAIN audit per metric even

## Layer 9 (5 FRs — buildable in parallel)

- **FR-INV-002** [MUST, 6h, slice 1] — INV multi-currency support — VND/USD/SGD/EUR/GBP with daily SBV FX snapshot + pe
- **FR-INV-007** [MUST, 6h, slice 2] — INV VN hóa đơn auto-emit on AM-send — Decree 123/2020 GDT XML signing + idempote
- **FR-INV-009** [MUST, 4h, slice 2] — INV AR aging report — current/30/60/90/120+ bucket rollup per customer + per eng
- **FR-INV-011** [MUST, 5h, slice 2] — INV revenue recognition — ASC 606 / IFRS 15 compliant deferred-revenue rollforwa
- **FR-TEN-003** [MUST, 8h, slice 2] — Stripe billing integration — USD/EUR/SGD/GBP customer + subscription + per-perio

## Layer 10 (5 FRs — buildable in parallel)

- **FR-CRM-010** [MUST, 5h, slice 7] — CRM vn-vat-invoice skill — Decree 123 hóa đơn auto-emit on deal.stage=won + invo
- **FR-INV-008** [MUST, 5h, slice 2] — INV VN hóa đơn cancellation flow — Decree 123 Art. 19 replacement-or-cancellatio
- **FR-INV-010** [MUST, 5h, slice 2] — INV CUO dunning draft — auto-generate polite/firm/legal-warning email drafts per
- **FR-TEN-101** [MUST, 10h, slice 1] — Self-serve signup form ≤ 30 s end-to-end — email OTP + slug + plan + currency + 
- **FR-TEN-102** [MUST, 12h, slice 2] — VND domestic billing rail — VnPay + Momo + ZaloPay subscription, recurring-charg

## Layer 11 (4 FRs — buildable in parallel)

- **FR-PORTAL-001** [MUST, 12h, slice 1] — PORTAL scoped read-only views — PROJ/INV/DOC/CHAT filtered by Engagement members
- **FR-PORTAL-002** [MUST, 8h, slice 1] — PORTAL per-tenant brand pack — logo + colour palette + custom CNAME + email temp
- **FR-TEN-107** [SHOULD, 16h, slice 3] — TEN tenant-admin SPA — seats + billing + audit + residency + retention dashboard
- **FR-TIME-008** [MUST, 8h, slice 2] — TIME expense capture — photo → AWS Textract OCR → hóa đơn parser → Member confir

## Layer 12 (2 FRs — buildable in parallel)

- **FR-PORTAL-007** [SHOULD, 6h, slice 2] — PORTAL PWA installable — mobile-first Progressive Web App with offline-capable v
- **FR-PORTAL-008** [MUST, 5h, slice 2] — PORTAL DSAR self-service — GDPR Art. 15 + PDPL Art. 17 client-initiated data sub

---

# §3 — Sprint plan (effort rollup)


_Generated 2026-05-17 — 241 FRs, 1791 total engineering-hours._

## Headline numbers

- **Total scope:** 241 FRs, 1791h (224 engineer-days @ 8h/d, or 11.2 engineer-months @ 160h/m).
- **At 3 engineers (480h/sprint @ 2-week sprints):** 3.7 sprints (~7.5 weeks).
- **At 5 engineers (800h/sprint):** 2.2 sprints (~4.5 weeks).

## By module

| Module | FRs | Total hours | Slices |
|---|---:|---:|---|
| **AI** | 23 | 175 | 1, 2, 3, 4, 5 |
| **AUTH** | 15 | 127 | 1 |
| **BRAIN** | 11 | 136 | 1, 2 |
| **CHAT** | 12 | 95 | 1, 2 |
| **CRM** | 10 | 52 | 1, 5, 6, 7 |
| **CUO** | 5 | 37 | 2, 6 |
| **DOC** | 11 | 103 | 1, 2, 3 |
| **DOCS** | 1 | 14 | 1 |
| **EMAIL** | 11 | 85 | 1, 2 |
| **ESOP** | 7 | 38 | 1, 2 |
| **HR** | 9 | 52 | 1, 6, 7 |
| **INV** | 11 | 67 | 1, 2 |
| **KB** | 9 | 49 | 1, 4, 5 |
| **LEARN** | 7 | 40 | 7 |
| **MCP** | 8 | 56 | 2, 3, 4 |
| **OBS** | 9 | 82 | 1, 2, 3 |
| **OKR** | 7 | 42 | 1, 3 |
| **PORTAL** | 8 | 61 | 1, 2 |
| **PROJ** | 18 | 128 | 1, 2, 3 |
| **RES** | 5 | 38 | 7, 8 |
| **REW** | 10 | 55 | 1, 2 |
| **SKILL** | 11 | 84 | 1, 2, 3 |
| **TEN** | 14 | 124 | 1, 2, 3 |
| **TIME** | 9 | 51 | 1, 2 |

## By module & slice (sprint chunks)

### AI

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 6 | 39 | FR-AI-001, FR-AI-002, FR-AI-003, FR-AI-004, FR-AI-005, FR-AI-104 |
| 2 | 5 | 34 | FR-AI-006, FR-AI-007, FR-AI-008, FR-AI-009, FR-AI-010 |
| 3 | 5 | 38 | FR-AI-011, FR-AI-012, FR-AI-013, FR-AI-014, FR-AI-015 |
| 4 | 5 | 42 | FR-AI-016, FR-AI-017, FR-AI-018, FR-AI-019, FR-AI-020 |
| 5 | 2 | 22 | FR-AI-021, FR-AI-022 |

### AUTH

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 15 | 127 | FR-AUTH-001, FR-AUTH-002, FR-AUTH-003, FR-AUTH-004, FR-AUTH-005, FR-AUTH-006, FR-AUTH-101, FR-AUTH-102, FR-AUTH-103, FR-AUTH-104, FR-AUTH-105, FR-AUTH-106, FR-AUTH-107, FR-AUTH-108, FR-AUTH-109 |

### BRAIN

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 4 | 52 | FR-BRAIN-101, FR-BRAIN-102, FR-BRAIN-103, FR-BRAIN-106 |
| 2 | 7 | 84 | FR-BRAIN-104, FR-BRAIN-105, FR-BRAIN-107, FR-BRAIN-108, FR-BRAIN-109, FR-BRAIN-110, FR-BRAIN-111 |

### CHAT

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 5 | 46 | FR-CHAT-001, FR-CHAT-002, FR-CHAT-003, FR-CHAT-004, FR-CHAT-005 |
| 2 | 7 | 49 | FR-CHAT-006, FR-CHAT-007, FR-CHAT-008, FR-CHAT-009, FR-CHAT-010, FR-CHAT-011, FR-CHAT-012 |

### CRM

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-CRM-001 |
| 5 | 3 | 18 | FR-CRM-002, FR-CRM-003, FR-CRM-004 |
| 6 | 3 | 16 | FR-CRM-005, FR-CRM-006, FR-CRM-007 |
| 7 | 3 | 12 | FR-CRM-008, FR-CRM-009, FR-CRM-010 |

### CUO

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 2 | 1 | 12 | FR-CUO-101 |
| 6 | 4 | 25 | FR-CUO-102, FR-CUO-103, FR-CUO-104, FR-CUO-105 |

### DOC

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 4 | 23 | FR-DOC-001, FR-DOC-007, FR-DOC-008, FR-DOC-009 |
| 2 | 2 | 18 | FR-DOC-005, FR-DOC-006 |
| 3 | 5 | 62 | FR-DOC-002, FR-DOC-003, FR-DOC-004, FR-DOC-010, FR-DOC-011 |

### DOCS

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 14 | FR-DOCS-001 |

### EMAIL

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 7 | 44 | FR-EMAIL-001, FR-EMAIL-002, FR-EMAIL-004, FR-EMAIL-006, FR-EMAIL-007, FR-EMAIL-009, FR-EMAIL-010 |
| 2 | 4 | 41 | FR-EMAIL-003, FR-EMAIL-005, FR-EMAIL-008, FR-EMAIL-011 |

### ESOP

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 14 | FR-ESOP-001, FR-ESOP-002, FR-ESOP-003 |
| 2 | 4 | 24 | FR-ESOP-004, FR-ESOP-005, FR-ESOP-006, FR-ESOP-007 |

### HR

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-HR-001 |
| 6 | 6 | 32 | FR-HR-002, FR-HR-003, FR-HR-004, FR-HR-005, FR-HR-006, FR-HR-007 |
| 7 | 2 | 14 | FR-HR-008, FR-HR-009 |

### INV

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 20 | FR-INV-001, FR-INV-002, FR-INV-004 |
| 2 | 8 | 47 | FR-INV-003, FR-INV-005, FR-INV-006, FR-INV-007, FR-INV-008, FR-INV-009, FR-INV-010, FR-INV-011 |

### KB

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-KB-001 |
| 4 | 2 | 10 | FR-KB-002, FR-KB-003 |
| 5 | 6 | 33 | FR-KB-004, FR-KB-005, FR-KB-006, FR-KB-007, FR-KB-008, FR-KB-009 |

### LEARN

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 7 | 7 | 40 | FR-LEARN-001, FR-LEARN-002, FR-LEARN-003, FR-LEARN-004, FR-LEARN-005, FR-LEARN-006, FR-LEARN-007 |

### MCP

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 2 | 5 | 28 | FR-MCP-002, FR-MCP-003, FR-MCP-004, FR-MCP-005, FR-MCP-006 |
| 3 | 2 | 16 | FR-MCP-007, FR-MCP-008 |
| 4 | 1 | 12 | FR-MCP-001 |

### OBS

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 30 | FR-OBS-001, FR-OBS-002, FR-OBS-003 |
| 2 | 3 | 20 | FR-OBS-004, FR-OBS-005, FR-OBS-006 |
| 3 | 3 | 32 | FR-OBS-007, FR-OBS-008, FR-OBS-009 |

### OKR

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 1 | 6 | FR-OKR-001 |
| 3 | 6 | 36 | FR-OKR-002, FR-OKR-003, FR-OKR-004, FR-OKR-005, FR-OKR-006, FR-OKR-007 |

### PORTAL

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 3 | 30 | FR-PORTAL-001, FR-PORTAL-002, FR-PORTAL-003 |
| 2 | 5 | 31 | FR-PORTAL-004, FR-PORTAL-005, FR-PORTAL-006, FR-PORTAL-007, FR-PORTAL-008 |

### PROJ

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 2 | 19 | FR-PROJ-001, FR-PROJ-002 |
| 2 | 7 | 41 | FR-PROJ-003, FR-PROJ-004, FR-PROJ-005, FR-PROJ-006, FR-PROJ-007, FR-PROJ-008, FR-PROJ-009 |
| 3 | 9 | 68 | FR-PROJ-010, FR-PROJ-011, FR-PROJ-012, FR-PROJ-013, FR-PROJ-014, FR-PROJ-015, FR-PROJ-016, FR-PROJ-017, FR-PROJ-018 |

### RES

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 7 | 1 | 10 | FR-RES-001 |
| 8 | 4 | 28 | FR-RES-002, FR-RES-003, FR-RES-004, FR-RES-005 |

### REW

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 5 | 25 | FR-REW-001, FR-REW-002, FR-REW-003, FR-REW-004, FR-REW-010 |
| 2 | 5 | 30 | FR-REW-005, FR-REW-006, FR-REW-007, FR-REW-008, FR-REW-009 |

### SKILL

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 6 | 46 | FR-SKILL-101, FR-SKILL-102, FR-SKILL-103, FR-SKILL-104, FR-SKILL-107, FR-SKILL-201 |
| 2 | 1 | 9 | FR-SKILL-105 |
| 3 | 4 | 29 | FR-SKILL-106, FR-SKILL-108, FR-SKILL-109, FR-SKILL-110 |

### TEN

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 7 | 60 | FR-TEN-001, FR-TEN-002, FR-TEN-004, FR-TEN-101, FR-TEN-104, FR-TEN-201, FR-TEN-202 |
| 2 | 6 | 48 | FR-TEN-003, FR-TEN-005, FR-TEN-102, FR-TEN-103, FR-TEN-105, FR-TEN-106 |
| 3 | 1 | 16 | FR-TEN-107 |

### TIME

| Slice | FRs | Hours | FR list |
|---|---:|---:|---|
| 1 | 7 | 37 | FR-TIME-001, FR-TIME-002, FR-TIME-003, FR-TIME-005, FR-TIME-006, FR-TIME-007, FR-TIME-009 |
| 2 | 2 | 14 | FR-TIME-004, FR-TIME-008 |

---

# §4 — Migration audit (per-module SQL)


_Generated 2026-05-17 — scanned all FR `build_envelope.new_files` for `services/<module>/migrations/<N>_<name>.sql` patterns._

## Summary

- Total modules with migrations: **23**
- Total migration files declared: **327**

### `ai`

- Total unique migrations: **1**
- Sequence range: `0010` → `0010`
- ⚠️ **Gaps in sequence**: [1, 2, 3, 4, 5, 6, 7, 8, 9]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0010 | `vn_provider_creds` | FR-AI-104 |

### `auth`

- Total unique migrations: **29**
- Sequence range: `0001` → `0026`
- ⚠️ **Gaps in sequence**: [8, 9]
- ⚠️ **Duplicate seq with different names**:
  - `0005`: ['rls_enable_on_tables', 'roles_permissions']
  - `0006`: ['role_catalogue_version', 'signing_keys']
  - `0015`: ['auth_token_refresh_log', 'hibp_audit']
  - `0016`: ['login_history_geo', 'mfa_factors']
  - `0017`: ['mfa_factor_history', 'travel_audit']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `tenants` | FR-AUTH-001 |
| 0002 | `admin_idempotency` | FR-AUTH-001 |
| 0003 | `subjects` | FR-AUTH-002 |
| 0004 | `rls_roles` | FR-AUTH-003 |
| 0005 | `rls_enable_on_tables` | FR-AUTH-003 |
| 0005 | `roles_permissions` | FR-AUTH-101 |
| 0006 | `role_catalogue_version` | FR-AUTH-101 |
| 0006 | `signing_keys` | FR-AUTH-004 |
| 0007 | `sessions` | FR-AUTH-005 |
| 0010 | `oidc_idp_configs` | FR-AUTH-104 |
| 0011 | `oidc_login_history` | FR-AUTH-104 |
| 0012 | `oidc_subject_link` | FR-AUTH-104 |
| 0013 | `lumi_token_issuance_log` | FR-AUTH-108 |
| 0014 | `auth_migration_state` | FR-AUTH-109 |
| 0015 | `auth_token_refresh_log` | FR-AUTH-109 |
| 0015 | `hibp_audit` | FR-AUTH-107 |
| 0016 | `login_history_geo` | FR-AUTH-106 |
| 0016 | `mfa_factors` | FR-AUTH-102 |
| 0017 | `mfa_factor_history` | FR-AUTH-102 |
| 0017 | `travel_audit` | FR-AUTH-106 |
| 0018 | `mfa_challenge_log` | FR-AUTH-102 |
| 0019 | `mfa_recovery_codes` | FR-AUTH-102 |
| 0020 | `mfa_lockout_state` | FR-AUTH-102 |
| 0021 | `saml_idp_configs` | FR-AUTH-103 |
| 0022 | `saml_login_history` | FR-AUTH-103 |
| 0023 | `saml_authn_request_log` | FR-AUTH-103 |
| 0024 | `saml_subject_link` | FR-AUTH-103 |
| 0025 | `passkey_enrolment_state` | FR-AUTH-105 |
| 0026 | `passkey_lifecycle_log` | FR-AUTH-105 |

### `brain`

- Total unique migrations: **3**
- Sequence range: `0001` → `0003`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `layer2` | FR-BRAIN-101 |
| 0002 | `layer2_cursor` | FR-BRAIN-101 |
| 0003 | `pgroonga` | FR-BRAIN-108 |

### `crm`

- Total unique migrations: **15**
- Sequence range: `0001` → `0010`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['activity_feed', 'contacts']
  - `0003`: ['pipelines_stages', 'vn_account_fields']
  - `0004`: ['deal_conversion', 'deals']
  - `0005`: ['deal_status_history', 'next_action_suggestions']
  - `0006`: ['lead_scoring', 'seed_pipelines']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `accounts` | FR-CRM-001 |
| 0002 | `activity_feed` | FR-CRM-002 |
| 0002 | `contacts` | FR-CRM-001 |
| 0003 | `pipelines_stages` | FR-CRM-001 |
| 0003 | `vn_account_fields` | FR-CRM-003 |
| 0004 | `deal_conversion` | FR-CRM-004 |
| 0004 | `deals` | FR-CRM-001 |
| 0005 | `deal_status_history` | FR-CRM-001 |
| 0005 | `next_action_suggestions` | FR-CRM-005 |
| 0006 | `lead_scoring` | FR-CRM-006 |
| 0006 | `seed_pipelines` | FR-CRM-001 |
| 0007 | `win_loss_drafts` | FR-CRM-007 |
| 0008 | `mst_validation` | FR-CRM-008 |
| 0009 | `tenant_bank_config` | FR-CRM-009 |
| 0010 | `vat_invoice_emissions` | FR-CRM-010 |

### `cuo`

- Total unique migrations: **4**
- Sequence range: `0002` → `0005`
- ⚠️ **Gaps in sequence**: [1]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0002 | `langgraph_checkpoints` | FR-CUO-102 |
| 0003 | `trace_rows` | FR-CUO-103 |
| 0004 | `chain_walks` | FR-CUO-104 |
| 0005 | `chain_rollbacks` | FR-CUO-105 |

### `doc`

- Total unique migrations: **12**
- Sequence range: `0001` → `0011`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['document_audit_log', 'lifecycle_metadata']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `document_metadata` | FR-DOC-001 |
| 0002 | `document_audit_log` | FR-DOC-001 |
| 0002 | `lifecycle_metadata` | FR-DOC-007 |
| 0003 | `expiry_alerts` | FR-DOC-008 |
| 0004 | `renewal_drafts` | FR-DOC-009 |
| 0005 | `identity_verifications` | FR-DOC-006 |
| 0006 | `signing_workflows` | FR-DOC-005 |
| 0007 | `third_party_imports` | FR-DOC-010 |
| 0008 | `qtsp_signatures` | FR-DOC-002 |
| 0009 | `aatl_signatures` | FR-DOC-003 |
| 0010 | `vn_ca_signatures` | FR-DOC-004 |
| 0011 | `ltv_operations` | FR-DOC-011 |

### `email`

- Total unique migrations: **16**
- Sequence range: `0001` → `0012`
- ⚠️ **Duplicate seq with different names**:
  - `0001`: ['email_auth_log', 'messages']
  - `0002`: ['bounce_log', 'tenant_dkim_keys']
  - `0003`: ['dkim_keys', 'tenant_dns_setup']
  - `0004`: ['outbound_messages', 'residency_routing']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `email_auth_log` | FR-EMAIL-002 |
| 0001 | `messages` | FR-EMAIL-001 |
| 0002 | `bounce_log` | FR-EMAIL-001 |
| 0002 | `tenant_dkim_keys` | FR-EMAIL-004 |
| 0003 | `dkim_keys` | FR-EMAIL-001 |
| 0003 | `tenant_dns_setup` | FR-EMAIL-004 |
| 0004 | `outbound_messages` | FR-EMAIL-009 |
| 0004 | `residency_routing` | FR-EMAIL-001 |
| 0005 | `suppression_list` | FR-EMAIL-009 |
| 0006 | `bulk_sends` | FR-EMAIL-010 |
| 0007 | `dsar_export_jobs` | FR-EMAIL-011 |
| 0008 | `tracked_domains` | FR-EMAIL-006 |
| 0009 | `message_issue_link` | FR-EMAIL-007 |
| 0010 | `genie_sessions` | FR-EMAIL-008 |
| 0011 | `camel_audit` | FR-EMAIL-005 |
| 0012 | `thread_state` | FR-EMAIL-003 |

### `esop`

- Total unique migrations: **7**
- Sequence range: `0001` → `0007`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `sp_grants` | FR-ESOP-001 |
| 0002 | `vesting_accruals` | FR-ESOP-002 |
| 0003 | `annual_valuations` | FR-ESOP-003 |
| 0004 | `put_options` | FR-ESOP-004 |
| 0005 | `leaver_outcomes` | FR-ESOP-005 |
| 0006 | `ma_events` | FR-ESOP-006 |
| 0007 | `dashboard_access_log` | FR-ESOP-007 |

### `hr`

- Total unique migrations: **11**
- Sequence range: `0001` → `0009`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['contract_types', 'member_status_history']
  - `0003`: ['cccd_storage', 'member_view']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `members` | FR-HR-001 |
| 0002 | `contract_types` | FR-HR-002 |
| 0002 | `member_status_history` | FR-HR-001 |
| 0003 | `cccd_storage` | FR-HR-003 |
| 0003 | `member_view` | FR-HR-001 |
| 0004 | `leave_requests` | FR-HR-004 |
| 0005 | `policy_constants` | FR-HR-005 |
| 0006 | `leave_accrual_ledger` | FR-HR-006 |
| 0007 | `perf_snapshots` | FR-HR-008 |
| 0008 | `terminations` | FR-HR-009 |
| 0009 | `onboarding_sagas` | FR-HR-007 |

### `inv`

- Total unique migrations: **14**
- Sequence range: `0001` → `0021`
- ⚠️ **Gaps in sequence**: [7, 8, 9, 16, 17, 18, 19]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `invoices` | FR-INV-001 |
| 0002 | `invoice_lines` | FR-INV-001 |
| 0003 | `invoice_status_history` | FR-INV-001 |
| 0004 | `invoice_number_sequence` | FR-INV-001 |
| 0005 | `rate_card_snapshot` | FR-INV-001 |
| 0006 | `fx_rates` | FR-INV-002 |
| 0010 | `payment_receipts` | FR-INV-005 |
| 0011 | `webhook_secrets` | FR-INV-005 |
| 0012 | `stripe_event_log` | FR-INV-003 |
| 0013 | `stripe_webhook_secrets` | FR-INV-003 |
| 0014 | `payment_allocations` | FR-INV-006 |
| 0015 | `invoice_outstanding_view` | FR-INV-006 |
| 0020 | `wise_webhook_events` | FR-INV-004 |
| 0021 | `wise_unmatched_receipts` | FR-INV-004 |

### `invoicing`

- Total unique migrations: **4**
- Sequence range: `0007` → `0010`
- ⚠️ **Gaps in sequence**: [1, 2, 3, 4, 5, 6]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0007 | `vn_hoadon` | FR-INV-007 |
| 0008 | `vn_hoadon_cancellation` | FR-INV-008 |
| 0009 | `dunning_drafts` | FR-INV-010 |
| 0010 | `recognition` | FR-INV-011 |

### `kb`

- Total unique migrations: **10**
- Sequence range: `0001` → `0009`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['document_views', 'render_cache']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `documents` | FR-KB-001 |
| 0002 | `document_views` | FR-KB-001 |
| 0002 | `render_cache` | FR-KB-002 |
| 0003 | `permissions_share_links` | FR-KB-003 |
| 0004 | `pgroonga_fts5_index` | FR-KB-004 |
| 0005 | `semantic_chunks` | FR-KB-005 |
| 0006 | `rerank_cache` | FR-KB-006 |
| 0007 | `qa_questions` | FR-KB-007 |
| 0008 | `runbook_tags` | FR-KB-008 |
| 0009 | `translation_link` | FR-KB-009 |

### `learn`

- Total unique migrations: **7**
- Sequence range: `0001` → `0007`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `skill_tree_mastery` | FR-LEARN-001 |
| 0002 | `evidence` | FR-LEARN-002 |
| 0003 | `vp_snapshots` | FR-LEARN-003 |
| 0004 | `councils` | FR-LEARN-004 |
| 0005 | `disclosure_log` | FR-LEARN-005 |
| 0006 | `promotions` | FR-LEARN-006 |
| 0007 | `vp_rew_handoffs` | FR-LEARN-007 |

### `mcp`

- Total unique migrations: **9**
- Sequence range: `0002` → `0012`
- ⚠️ **Gaps in sequence**: [1, 3, 4]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0002 | `server_heartbeats` | FR-MCP-002 |
| 0005 | `prm_drift_log` | FR-MCP-005 |
| 0006 | `mcp_gating_policy` | FR-MCP-006 |
| 0007 | `mcp_pending_confirmations` | FR-MCP-006 |
| 0008 | `mcp_gating_decisions_log` | FR-MCP-006 |
| 0009 | `mcp_tasks` | FR-MCP-007 |
| 0010 | `mcp_task_checkpoints` | FR-MCP-007 |
| 0011 | `mcp_task_progress_events` | FR-MCP-007 |
| 0012 | `mcp_elicitations` | FR-MCP-008 |

### `metering`

- Total unique migrations: **4**
- Sequence range: `0001` → `0003`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['metering_holds_index', 'metering_periods']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `metering_events` | FR-TEN-004 |
| 0002 | `metering_holds_index` | FR-TEN-004 |
| 0002 | `metering_periods` | FR-TEN-004 |
| 0003 | `metering_aggregates_view` | FR-TEN-004 |

### `okr`

- Total unique migrations: **12**
- Sequence range: `0001` → `0007`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['kr_types', 'teams']
  - `0003`: ['objectives', 'progress_source']
  - `0004`: ['auto_progress_runs', 'key_results']
  - `0005`: ['progress_log', 'weekly_checkins']
  - `0006`: ['monday_digests', 'objective_status_history']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `cycles` | FR-OKR-001 |
| 0002 | `kr_types` | FR-OKR-002 |
| 0002 | `teams` | FR-OKR-001 |
| 0003 | `objectives` | FR-OKR-001 |
| 0003 | `progress_source` | FR-OKR-003 |
| 0004 | `auto_progress_runs` | FR-OKR-004 |
| 0004 | `key_results` | FR-OKR-001 |
| 0005 | `progress_log` | FR-OKR-001 |
| 0005 | `weekly_checkins` | FR-OKR-005 |
| 0006 | `monday_digests` | FR-OKR-006 |
| 0006 | `objective_status_history` | FR-OKR-001 |
| 0007 | `quarterly_retros` | FR-OKR-007 |

### `portal`

- Total unique migrations: **21**
- Sequence range: `0001` → `0021`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `portal_idp_configs` | FR-PORTAL-003 |
| 0002 | `portal_scim_audit_log` | FR-PORTAL-003 |
| 0003 | `portal_idp_groups_map` | FR-PORTAL-003 |
| 0004 | `portal_scim_tokens` | FR-PORTAL-003 |
| 0005 | `portal_brand_packs` | FR-PORTAL-002 |
| 0006 | `portal_brand_pack_active` | FR-PORTAL-002 |
| 0007 | `portal_brand_assets` | FR-PORTAL-002 |
| 0008 | `portal_cname_configs` | FR-PORTAL-002 |
| 0009 | `portal_deprovision_log` | FR-PORTAL-004 |
| 0010 | `portal_jwt_blacklist` | FR-PORTAL-004 |
| 0011 | `portal_restore_requests` | FR-PORTAL-004 |
| 0012 | `portal_genie_sessions` | FR-PORTAL-005 |
| 0013 | `portal_genie_messages` | FR-PORTAL-005 |
| 0014 | `portal_view_definitions` | FR-PORTAL-001 |
| 0015 | `portal_view_read_log` | FR-PORTAL-001 |
| 0016 | `portal_dsar_requests` | FR-PORTAL-008 |
| 0017 | `portal_dsar_denials` | FR-PORTAL-008 |
| 0018 | `portal_workflow_submissions` | FR-PORTAL-006 |
| 0019 | `portal_workflow_routing_rules` | FR-PORTAL-006 |
| 0020 | `portal_pwa_subscriptions` | FR-PORTAL-007 |
| 0021 | `portal_pwa_notifications_log` | FR-PORTAL-007 |

### `proj`

- Total unique migrations: **5**
- Sequence range: `0001` → `0010`
- ⚠️ **Gaps in sequence**: [5, 6, 7, 8, 9]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `engagements` | FR-PROJ-001 |
| 0002 | `cycles` | FR-PROJ-001 |
| 0003 | `issues` | FR-PROJ-001 |
| 0004 | `issue_links` | FR-PROJ-001 |
| 0010 | `issues_addendum` | FR-TIME-001 |

### `res`

- Total unique migrations: **5**
- Sequence range: `0001` → `0005`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `capacity_demand_matrix` | FR-RES-001 |
| 0002 | `allocation_changes` | FR-RES-002 |
| 0003 | `allocation_flags` | FR-RES-003 |
| 0004 | `hiring_memos` | FR-RES-004 |
| 0005 | `ot_consent` | FR-RES-005 |

### `rew`

- Total unique migrations: **9**
- Sequence range: `0001` → `0009`
- ✓ Sequence clean

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `comp_schema` | FR-REW-001 |
| 0002 | `param_versions` | FR-REW-002 |
| 0003 | `p1_protection` | FR-REW-003 |
| 0004 | `deductions` | FR-REW-004 |
| 0005 | `payroll_runs` | FR-REW-005 |
| 0006 | `payslip_pdfs` | FR-REW-006 |
| 0007 | `bp_ledger` | FR-REW-007 |
| 0008 | `p3_distributions` | FR-REW-008 |
| 0009 | `payroll_batches` | FR-REW-009 |

### `skill`

- Total unique migrations: **1**
- Sequence range: `0010` → `0010`
- ⚠️ **Gaps in sequence**: [1, 2, 3, 4, 5, 6, 7, 8, 9]

| Seq | Name | Declaring FR |
|---:|---|---|
| 0010 | `oci_bundles` | FR-SKILL-201 |

### `ten`

- Total unique migrations: **33**
- Sequence range: `0001` → `0029`
- ⚠️ **Duplicate seq with different names**:
  - `0004`: ['plan_tier', 'tenant_offboarding_state']
  - `0005`: ['plan_history', 'tenant_offboarding_log']
  - `0010`: ['holdco_flips', 'stripe_price_map']
  - `0011`: ['hostile_overrides', 'signup_sessions']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `tenants` | FR-TEN-001 |
| 0002 | `tenant_status_history` | FR-TEN-001 |
| 0003 | `tenant_residency_map` | FR-TEN-001 |
| 0004 | `plan_tier` | FR-TEN-002 |
| 0004 | `tenant_offboarding_state` | FR-TEN-104 |
| 0005 | `plan_history` | FR-TEN-002 |
| 0005 | `tenant_offboarding_log` | FR-TEN-104 |
| 0006 | `stripe_billing` | FR-TEN-003 |
| 0007 | `stripe_api_calls` | FR-TEN-003 |
| 0008 | `stripe_event_dispatch_log` | FR-TEN-003 |
| 0009 | `billing_currency_enum` | FR-TEN-003 |
| 0010 | `holdco_flips` | FR-TEN-201 |
| 0010 | `stripe_price_map` | FR-TEN-003 |
| 0011 | `hostile_overrides` | FR-TEN-202 |
| 0011 | `signup_sessions` | FR-TEN-101 |
| 0012 | `tenant_consents` | FR-TEN-101 |
| 0013 | `signup_rate_limits` | FR-TEN-101 |
| 0014 | `disposable_email_domains` | FR-TEN-101 |
| 0015 | `residency_enum` | FR-TEN-103 |
| 0016 | `residency_trip_wire` | FR-TEN-103 |
| 0017 | `residency_health_log` | FR-TEN-103 |
| 0018 | `vnd_payment_tokens` | FR-TEN-102 |
| 0019 | `vnd_psp_credentials` | FR-TEN-102 |
| 0020 | `vnd_invoices` | FR-TEN-102 |
| 0021 | `vnd_invoice_sequence` | FR-TEN-102 |
| 0022 | `vnd_event_dispatch_log` | FR-TEN-102 |
| 0023 | `vertical_pack_installs` | FR-TEN-005 |
| 0024 | `vertical_pack_price_catalog` | FR-TEN-005 |
| 0025 | `vertical_pack_overrides` | FR-TEN-005 |
| 0026 | `tenant_bundle_exports` | FR-TEN-105 |
| 0027 | `tenant_signing_keys` | FR-TEN-105 |
| 0028 | `permanent_delete_attestations` | FR-TEN-106 |
| 0029 | `permanent_delete_cascade_log` | FR-TEN-106 |

### `time`

- Total unique migrations: **11**
- Sequence range: `0001` → `0010`
- ⚠️ **Duplicate seq with different names**:
  - `0002`: ['time_entries_view', 'timers']

| Seq | Name | Declaring FR |
|---:|---|---|
| 0001 | `time_entries` | FR-TIME-001 |
| 0002 | `time_entries_view` | FR-TIME-001 |
| 0002 | `timers` | FR-TIME-002 |
| 0003 | `vn_ot_tracking` | FR-TIME-007 |
| 0004 | `billable_defaults` | FR-TIME-005 |
| 0005 | `timesheets` | FR-TIME-006 |
| 0006 | `timesheet_reviews` | FR-TIME-006 |
| 0007 | `rollup_cache` | FR-TIME-009 |
| 0008 | `time_proposals` | FR-TIME-004 |
| 0009 | `expenses` | FR-TIME-008 |
| 0010 | `expense_policies` | FR-TIME-008 |

---

**Total issues found:** 18

**Interpretation**: Gaps may indicate planned but un-numbered migrations; duplicates with different names indicate two FRs claim the same sequence (must reconcile).