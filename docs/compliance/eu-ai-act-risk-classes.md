# EU AI Act risk classes — bucket-selection rules

> Referenced from `docs/templates/feature_request.md` and `docs/CONTRIBUTING.md`.
>
> Source of authority: EU AI Act (Regulation (EU) 2024/1689). Articles 5, 6, 7, 14, 50, and Annex III. The PRD/SRS map specific CyberOS features to these buckets; this document is the bucket-selection rulebook.

---

## The four buckets

The `eu_ai_act_risk_class` frontmatter field accepts exactly these values:

| Value | What it means | Validator behaviour |
|---|---|---|
| `not_ai` | The feature has no AI involvement at all. | No AI Risk Assessment required. |
| `minimal` | AI is involved but in a way that does not affect user-visible behaviour or generate content visible to a natural person. Examples: server-side anomaly detection used only for engineering alerts; spam filtering on outbound email. | No AI Risk Assessment required. Be honest — see anti-patterns below. |
| `limited` | The feature emits AI-generated content visible to a natural person, or affects user-visible behaviour through a model. Article 50 transparency obligation applies. | **AI Risk Assessment required** (Data Sources, Human Oversight, Failure Modes). |
| `high` | The feature falls under Annex III. For CyberOS the live triggers are Annex III §4 (employment, vocational training, promotion, dismissal, payroll, equity allocation, performance evaluation). | **AI Risk Assessment required** + EU AI Act Conformity Pack (SRS §6.8). |

`unacceptable` is **not** an allowed value. The schema rejects it. Features that fall under Article 5 prohibitions (social scoring, real-time biometric ID in public spaces for law enforcement, exploitation of vulnerabilities, etc.) must not be filed.

---

## Decision tree

```
Does the feature involve any model inference at all?
├── No → not_ai
└── Yes
    │
    Is the feature in {employment, training, promotion,
    dismissal, payroll, equity, performance evaluation}?
    │   These are CyberOS modules: REW, LEARN, ESOP, parts of HR.
    │
    ├── Yes → high  (Annex III §4)
    └── No
        │
        Does the feature emit AI-generated content visible
        to a natural person, or change user-visible behaviour
        through a model decision?
        │
        ├── Yes → limited  (Article 50)
        └── No  → minimal
```

---

## CyberOS module → default bucket

The generator (`scripts/gen-features.ts`) seeds `eu_ai_act_risk_class` from the FR's module unless the YAML overrides it explicitly:

| Module | Default bucket | Rationale |
|---|---|---|
| AUTH, MCP, OBS, PROJ, TIME, CRM, KB, EMAIL, INV, RES, OKR, DOC, CP | `not_ai` | No model inference in their critical path. |
| HR | `not_ai` (most FRs); `high` for FRs that drive performance, hiring, dismissal | Annex III §4 — set per-FR. |
| AI, MCP-AI, CHAT (with Genie surfaces) | `limited` | AI gateway / mascot surfaces emit AI content. |
| BRAIN | `limited` | RAG output is AI-derived content visible to users. |
| GENIE | `limited` | Mascot is the canonical user-facing AI surface. |
| REW | `high` | Payroll, equity → Annex III §4. Even though the math path is deterministic (DEC-029), explanations and AI-touching surfaces sit under high-risk. |
| LEARN | `high` | Career path / promotion / training → Annex III §4. |
| ESOP | `high` | Equity allocation → Annex III §4. |

These are **defaults**, not assertions. Set the right bucket per FR in `tasks.yaml` and let the validator confirm the required-when sections are present.

---

## What good looks like (limited bucket example)

`FR-GENIE-007` — "Genie suggests a follow-up message in CHAT":

- **Data Sources:** RAG over the user's own tenant data via BRAIN; no cross-tenant training; suggestions use the AI Gateway's primary model (DEC-018).
- **Human Oversight:** The suggestion is a draft inside the composer; the user must press send. No auto-send.
- **Failure Modes:** If the model is offline the composer shows no suggestion and the message flow is unchanged. If the suggestion contains a flagged term the composer presents a warning and does not pre-fill.

## What good looks like (high bucket example)

`FR-LEARN-009` — "AI explains the gap between current level and target level":

- **Data Sources:** the user's own LEARN history + the parameter-versioned competency rubric. No cross-tenant training. No compensation values.
- **Human Oversight:** the explanation is shown to the Member only; manager-facing summary uses the deterministic rubric, not the LLM. Promotion decisions never rely solely on the AI output (Article 14).
- **Failure Modes:** if the model is offline the deterministic rubric still produces a level + gap report; the AI explanation is omitted.
- **Conformity Pack hooks:** see SRS §6.8 — model card, evaluation report, post-market monitoring, log retention, fundamental-rights impact assessment.

---

## Anti-patterns

- **Picking `minimal` to dodge the AI Risk Assessment** when the feature actually shows AI-generated content to a user. The bucket is `limited`. The work is one paragraph per subsection — much cheaper than discovering the gap during an audit.
- **Picking `limited` for a payroll-impacting AI surface** to dodge the Conformity Pack. Annex III §4 is hard-coded — payroll, equity, promotion, dismissal — these are `high`. The Conformity Pack is real work; that is by design.
- **Picking `not_ai` for a feature whose UI clearly says "AI suggests…".** The schema can't catch this; review will.

---

## Cross-references

- [SRS §6 AI Integration Architecture](../SRS.md) — pipeline, latency budgets, model contracts.
- [SRS §6.7 AI Compliance Primitives](../SRS.md) — the seven primitives that satisfy multiple jurisdictions simultaneously.
- [SRS §6.8 EU AI Act Annex III §4 High-Risk Conformity Pack](../SRS.md) — the explicit pack required for `high` features in REW + LEARN.
- [PRD §10 Compliance & Trust Strategy](../PRD.md) — the tier model, cert sequence, and decline list.
