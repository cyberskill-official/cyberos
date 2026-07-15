# `_template/` — canonical skeleton for new skills

This directory ships the reusable scaffold every new author/audit skill pair copies from. Copy `_template/author/` into `skill/<your-artifact>-author/` and `_template/audit/` into `skill/<your-artifact>-audit/`, then fill in the artifact-specific bits.

## Step-by-step

### 1. Pick a name

Skill names use `kebab-case`, lowercase, `[a-z0-9-]+`. Follow `<artifact>-author` / `<artifact>-audit` for the author/audit pair convention. Examples already in the catalog: `statement-of-work-author`, `statement-of-work-audit`, `task-author`, `task-audit`, `product-requirements-document-author`, `product-requirements-document-audit`.

### 2. Copy the scaffold

```bash
cd skill
cp -r _template/author <artifact>-author
cp -r _template/audit  <artifact>-audit
```

### 3. Fill in the author skill

Edit `skill/<artifact>-author/`:

- `SKILL.md` — replace every `<ARTIFACT>` and `<artifact>` placeholder. Update `description`, `expects.required_fields`, `produces.output_kind`, `depends_on_contracts`.
- `INVARIANTS.md` — list the artifact-specific invariants the skill must hold.
- `PIPELINE.md` — describe how the skill produces its artifact and chains to the audit sibling.
- `STANDALONE_INTERVIEW.md` — the questions the skill asks when called without an envelope.
- `HUMAN_SUMMARY.md` — what the user sees in chat after each batch.
- `references/ANTI_FABRICATION.md`, `references/UNTRUSTED_CONTENT.md`, `references/HITL_PROTOCOL.md`, `references/FAILURE_MODES.md`, `references/MANIFEST_SCHEMA.md` — copy verbatim (or symlink) from the template; customize HITL categories and MANIFEST_SCHEMA only.
- `envelopes/input.json`, `envelopes/output.json` — define the JSON Schema for the skill's input/output.
- `acceptance/README.md` — describe the golden fixtures.
- `CHANGELOG.md` — start at `1.0.0 — initial author skill`.

### 4. Fill in the audit skill

Edit `skill/<artifact>-audit/`:

- `SKILL.md` — replace placeholders. The `description` SHOULD declare the rubric version it implements.
- `INVARIANTS.md` — include the `deterministic_drift` check by default.
- `RUBRIC.md` — the artifact-specific audit rules. Follow `../task-audit/RUBRIC.md`. Use stable `rule_id` strings.
- `REPORT_FORMAT.md` — the on-disk shape of `<artifact>.audit.md` reports.
- `AUDIT_LOOP.md` — usually a one-liner pointing at the canonical `skill/../task-audit/AUDIT_LOOP.md`.
- `PIPELINE.md`, `HUMAN_SUMMARY.md`, references (same as author), `envelopes/`, `acceptance/`, `CHANGELOG.md`.

### 5. Audit your work

Run your new audit skill against your new author skill's output (a sample artifact). Iterate until 10/10 (per the task-authoring loop discipline). Then add an entry to `MODULE.md` §3 marking the pair `shipped`.

### 6. Wire the chain

If your skill chains naturally to a downstream skill, set `expects.optional_fields.chain_to` default in the author's frontmatter. The CUO supervisor reads `produces.next_skill_recommendation` from the output envelope and queues the next skill unless the user opts out.

## What lives where

### `_template/author/`

| File | Purpose |
|---|---|
| `SKILL.md` | Frontmatter + body. The Anthropic Agent Skills entry point. |
| `INVARIANTS.md` | Self-audit invariants (confidence_low_streak, user_correction_streak, etc.). |
| `PIPELINE.md` | How the skill produces its artifact and chains to the audit sibling. |
| `STANDALONE_INTERVIEW.md` | Questions for chat-mode use (no envelope). |
| `HUMAN_SUMMARY.md` | Per-batch human-readable summary format. |
| `CHANGELOG.md` | Versioned skill changes. |
| `envelopes/input.json` | JSON Schema for input envelope. |
| `envelopes/output.json` | JSON Schema for output envelope. |
| `references/ANTI_FABRICATION.md` | Source-grounded discipline. |
| `references/UNTRUSTED_CONTENT.md` | Wrapping discipline + injection-marker scan. |
| `references/HITL_PROTOCOL.md` | `HITL_BATCH_REQUEST` format. |
| `references/FAILURE_MODES.md` | BOOT-001..008 catalog. |
| `references/MANIFEST_SCHEMA.md` | `manifest.json` shape for multi-artifact batches. |
| `acceptance/README.md` | Golden fixture catalog. |

### `_template/audit/`

| File | Purpose |
|---|---|
| `SKILL.md` | Frontmatter + body. Declares the rubric version it implements. |
| `INVARIANTS.md` | Self-audit invariants (includes `deterministic_drift` by default). |
| `AUDIT_LOOP.md` | Pointer to the canonical `skill/../task-audit/AUDIT_LOOP.md`. |
| `RUBRIC.md` | Per-artifact audit rules (FM/SEC/COND/QA/SAFE/STALE families). |
| `REPORT_FORMAT.md` | `.audit.md` frontmatter + per-issue block format. |
| `PIPELINE.md` | How the skill chains in (from author or directly) and what it emits downstream. |
| `HUMAN_SUMMARY.md` | Per-batch human-readable summary format. |
| `CHANGELOG.md` | Versioned skill changes. |
| `envelopes/input.json` | JSON Schema for input envelope. |
| `envelopes/output.json` | JSON Schema for output envelope. |
| `references/*` | Same as author (copy verbatim). |
| `acceptance/README.md` | Golden fixture catalog. |

## Anti-patterns — do not do these

- **Do not** reference sibling bundles for prompt content, rubric content, or fixtures. Each bundle is self-contained.
- **Do not** invent new top-level frontmatter fields. CyberOS extensions go under `metadata.cyberos-*` or in sibling `.md` files (INVARIANTS, RUBRIC, etc.).
- **Do not** ship an author without a matching audit. The pair is the unit of release.
- **Do not** ship a skill below 10/10 on its own rubric. Use HITL escalation if the rubric needs human input.
- **Do not** edit `_template/` for project-specific needs. If a future skill needs a different scaffold, propose a new template directory (e.g. `_template-chat/`).

## Cross-references

- `MODULE.md` — the canonical catalog this template feeds into.
- `../task-audit/AUDIT_LOOP.md` — the algorithm every audit skill implements.
- `../task-audit/RUBRIC.md` — the rubric format every audit skill follows.
- `../docs/appendices.md` — the Anthropic Agent Skills contract every skill bundle satisfies.
