# Security Policy

## Reporting a vulnerability

If you discover a security vulnerability in CyberOS, please report it responsibly.

- **Email:** `security@cyberskill.world` (or `info@cyberskill.world` until the dedicated alias is provisioned).
- **PGP key:** TBD — published in the Trust Center at `cyberos.world/trust` once issued (FR-GTM-001).
- **Disclosure window:** 90 days from initial report. CyberSkill will acknowledge within 72 hours and provide a fix timeline within 14 days.

Do **not** open public GitHub issues for security reports.

## Scope

In scope:

- All code and services in this repository.
- Production deployments at `*.cyberos.world` and `api.cyberos.world`.
- The MCP servers and AI Gateway components.

Out of scope:

- Customer-deployed integrations using the public API (FR-API-001, FR-API-002) — those are the customer's responsibility.
- Non-CyberSkill-managed third-party services (Bedrock, Stripe, Cloudflare, etc.) — report to the upstream provider.
- Social engineering of CyberSkill staff.
- Physical attacks on CyberSkill infrastructure.

## Bounty

A formal bug-bounty program will launch within 6 months of P4 GA (per FR-GTM-002 OQ-GTM-002-01). Until then, we offer public credit + thanks for valid reports.

## Production architectural commitments

CyberOS is built to satisfy the following structural security commitments. Any vulnerability that breaks one of these is treated as critical:

1. **No cross-tenant data access.** Postgres RLS + per-tenant KMS keys + cross-tenant invariant test harness (FR-TEN-001).
2. **No persistence of compensation, equity, government IDs, or bank accounts in BRAIN.** Ingestion-side denylist (DEC-036).
3. **Append-only Merkle-chained audit log.** Tampering detectable via external CLI verifier (FR-AUTH-002).
4. **AI never writes financial data.** Architectural rule (FR-REW-001, FR-INV-002).
5. **MCP persona-scope contracts enforced at three layers** — AI Gateway, MCP Gateway, module servers (FR-MCP-001).
6. **CaMeL dual-LLM anti-injection** for incoming user-content channels (FR-EMAIL-003, FR-CHAT-001).
7. **Crypto-shred on tenant deletion.** Per-tenant KMS keys destroyed; data is unrecoverable (FR-TEN-002).
8. **No payment instrument data in PORTAL.** Hosted-checkout deep-links only; PCI scope = SAQ A (FR-PORTAL-002).

## Compliance regimes

CyberOS is being audited for PDPL Decree 13/2023, GDPR, EU AI Act (Articles 5-7, 9-15, 50, 43, 47), ISO/IEC 27001 (P3), SOC 2 Type I (P3) → Type II (P4), ISO/IEC 42001 (P4), eIDAS QTSP usage, and Decree 130/2018 (VN e-signature). Trust Center publishes the canonical artefacts.

---

*Turn Your Will Into Real — securely.*
