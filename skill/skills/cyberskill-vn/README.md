# cyberskill-vn — Vietnamese-market skill collection

A curated bundle of CyberOS skills targeting Vietnam-specific workflows: tax codes (MST), VAT invoices, e-invoice XML generation, and more to come.

## What's here

| Skill | Purpose |
|---|---|
| `vn-mst-validate` | Validate Vietnamese tax codes (Mã số thuế) — structural check |
| `vn-vat-invoice`  | Generate GDT-schema VAT invoices from JSON line-items |

## Coming next

| Skill | Status |
|---|---|
| `vneid-integration` | VNeID identity verification API wrapper |
| `vn-bank-transfer`  | Napas247 + QR code transfer payload generators |
| `vn-legal-compliance` | Vietnamese legal compliance reference (Decree 13, Decree 53, etc.) |

## Strategic posture

This collection is the **CyberSkill differentiation play** per the architectural audit (`../../docs/AUDIT.md` §7 strategic addendum). Vietnam-localised skills are scarce in the global Agent Skills ecosystem. By publishing them to `agentskills.io`, CyberSkill becomes the de-facto provider of Vietnamese-market agentic capabilities across Claude Code, OpenAI Codex CLI, Cursor, VS Code + GitHub Copilot, Goose, Amp, Gemini CLI, and 20+ other compliant clients.

## Install

Per skill:

```bash
# From a local CyberOS host
cyberos-skill install file://./skill/skills/cyberskill-vn/vn-vat-invoice

# Or from the published registry (Phase 5+)
cyberos-skill install agentskills.io/cyberskill/vn-vat-invoice@0.1.0
```

## Contribute

CyberSkill maintains the bundle. PRs welcome at the CyberOS repo. New Vietnamese-market skills must:

1. Conform to the SKILL.md open spec (`../../docs/SPEC.md`)
2. Pass `cyberos-skill validate` on first contact
3. Carry `metadata.region = VN` and `metadata.collection = cyberskill-vn`
4. Be Apache-2.0 licensed (or MIT) for ecosystem compatibility
