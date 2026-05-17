# Anti-fabrication discipline

Version: 1.0.0  Status: Normative for every skill in the SKILL module.

This file is copied verbatim into every skill bundle. Customize only if the skill has a domain-specific fabrication risk worth calling out (e.g. `threat-model-author` may add "MUST NOT invent CVE IDs").

---

## §1  Core rules

§1.1  **Source-grounded claims only.** Every claim in an emitted artefact traces back to (a) a line in the source spec, (b) a BRAIN `memory_id`, or (c) a documented inference whose derivation is shown. No floating claims.

§1.2  **Authority markers required.** Every paragraph carries an `authority` field — one of `human-edited`, `human-confirmed`, `llm-explicit`, `llm-implicit` per AGENTS.md §5.1. Use the in-band marker syntax `<!-- authority: llm-explicit -->` at the end of the paragraph, or the structured `authority:` field if the artefact has a JSON-Schema-defined frontmatter that includes it.

§1.3  **HITL on ambiguity.** When the model cannot determine a field's value from sources alone, pause with `needs_human: true` and a precise question. Do not guess. Do not fill with placeholders unless the rubric explicitly allows TODO skeletons.

§1.4  **Untrusted-content wrapping.** Every quote of operator-supplied text is wrapped in `<untrusted_content source="<path>" page="<N|null>">…</untrusted_content>` blocks per AGENTS.md §11. The block boundaries MUST NOT be omitted, even for short quotes.

§1.5  **No fabricated identifiers.** Cross-references (ticket IDs, FR IDs, ADR IDs, person handles, dates) MUST resolve to real entities. If an identifier doesn't resolve, pause with HITL instead of inventing one.

§1.6  **No fabricated metrics.** Estimates and targets (numeric goals, deadlines, percentages, currency amounts) MUST cite a source. If no source exists, surface the gap as a HITL question (category: `success_metric_targets`).

§1.7  **No fabricated quotes.** Customer quotes, internal commentary, named-person statements MUST be quoted verbatim from a source. Wrapping in `<untrusted_content>` is required. If a quote is paraphrased, mark it `paraphrased: true` in the artefact metadata.

---

## §2  Forbidden practices

The skill MUST NEVER:

- Invent named entities (people, companies, products, places, events).
- Auto-set `eu_ai_act_risk_class` to `minimal` or `not_ai` when a determining fact is missing.
- Set `ai_authorship: none` on output the skill itself produced.
- Generate code, configuration, or API payloads not present in the source.
- Cite a URL that was not in the source or BRAIN.
- Cite a memory_id that does not exist in the current BRAIN.
- Cite a date past the configured `knowledge_cutoff_date` without flagging it as `extrapolated`.

---

## §3  Required attribution

Every emitted artefact carries:

- `source_ref:` field pointing at the line(s) in the source spec that justified its existence.
- Authority marker per claim (`authority: human-confirmed | llm-explicit | llm-implicit`).
- `provenance:` block on the artefact-level frontmatter declaring the source path + content SHA256 at read time.

This satisfies AGENTS.md §5.1 (authority hierarchy) and §9.1 (source-tier ordering) requirements.

---

## §4  Detection (for audit skills)

The matching audit skill (`product-roadmap-audit`) checks for fabrication via:

- `QA-CITE-001` — any claim without a `source_ref` → error.
- `QA-AUTH-001` — any paragraph without an `authority` marker → error.
- `QA-PROV-001` — missing `provenance` block on the artefact → error.
- `QA-NUM-001` — any numeric target without a citable source → error → needs_human.
- `QA-QUOTE-001` — any quoted text outside an `<untrusted_content>` block → warning.

See the audit skill's `RUBRIC.md` for the exact rule set.

---

## §5  When this discipline is hard

When the skill is asked to author from sparse input (e.g. a 50-word brief), the *correct* behaviour is to:

1. Read the brief.
2. Identify every artefact field that cannot be derived from the brief alone.
3. Surface those fields as HITL questions in a single batch.
4. Wait for human reply.
5. Author with the answered values.

The *incorrect* behaviour is to fill in plausible-sounding values. Plausible-sounding is the failure mode anti-fabrication exists to prevent.

---

## §6  Cross-references

- AGENTS.md §5.1, §9.1, §11 (memory module) — authority hierarchy, source-tier ordering, untrusted-content rules.
- `references/UNTRUSTED_CONTENT.md` (sibling file) — wrapping discipline + injection-marker scan.
- `references/HITL_PROTOCOL.md` (sibling file) — how to surface HITL questions.
- The matching audit skill's `RUBRIC.md` — concrete rule IDs that enforce this discipline.
