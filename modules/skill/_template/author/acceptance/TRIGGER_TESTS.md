---
skill_id: <artefact>-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for <artefact>-author

> **Purpose:** assert which user phrasings MUST route to this skill (positive) and which MUST NOT route here (negative). Per FR-SKILL-112, every production skill (`status: accepted` or higher) carries this file. The CUO supervisor's `classify_act` node is the unit under test.
>
> **Authoring tip:** mine real OBS logs when available (see `## Authoring notes`); otherwise use author intuition + Anthropic-guide-style paraphrasing. Aim for ≥3 positive + ≥3 negative. Phrases must be paraphrase-distinct (edit-distance > 3 — not single-character variants).

## Positive triggers (MUST route here)

- "Draft a <ARTIFACT> from this source"
- "Turn this <input> into a <ARTIFACT>"
- "Generate the <ARTIFACT> backlog from these docs"

## Negative triggers (MUST NOT route here)

- "Audit this existing <ARTIFACT>" → <artefact>-audit
- "Check the rubric on this <ARTIFACT>" → <artefact>-audit
- "What's our company holiday schedule?" → none

## Authoring notes

- Source-attribute every trigger: (a) real OBS phrasing observed during pilot week YYYY-WW, (b) Anthropic-guide-paraphrased, (c) author intuition.
- Edit-distance < 4 between two positives = duplicate (FM-113 rejects).
- Negative triggers fall into three pools per FR-SKILL-112 §1 #6: sibling-routing (a), cross-persona (b), no-skill (c — `→ none`).
- Re-author when `classifier_version` MAJOR-bumps.
