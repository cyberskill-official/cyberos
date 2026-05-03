# skills/

Anthropic Skills format directories for every CyberOS persona. Each `SKILL.md` is **dual-signed** (Founder + Engineering Lead) per FR-GENIE-001 before deployment to production.

## Layout

```
skills/
├── cuo/
│   ├── ceo/SKILL.md           # CUO/CEO strategic peer
│   ├── coo/SKILL.md           # CUO/COO operations peer
│   └── cto/SKILL.md           # CUO/CTO engineering peer
├── chro/SKILL.md              # Chief Human Resources (P1+)
├── cfo/SKILL.md               # Chief Financial (P2+)
├── cro/SKILL.md               # Chief Revenue (P1+)
├── clo/SKILL.md               # Chief Legal (P3+)
├── cso-strategy/SKILL.md      # Chief Strategy (P2+)
├── caio/SKILL.md              # Chief AI (emergent, P2-P3)
├── cxo/SKILL.md               # Chief Experience (emergent, P3-P4)
└── cso-sus/SKILL.md           # Chief Sustainability (emergent, P2-P3)
```

## Versioning

Every persona deployment emits a `persona_version` + `skill_version` chain that is stamped onto every AI response (DEC-029 + FR-AI-001). The chain is auditable (FR-AUTH-002) and surfaces in the tenant admin console (FR-TEN-005) + Trust Center (FR-GTM-001).

## Status

`stub` — created 2026-05-03. Real Skills authored as the GENIE work begins (P0 sprints).
