---
id: TASK-SKILL-115
title: "Sweep stale `<placeholder>` syntax in 134 production SKILL.md files (metadata.stage + description + allowed_memory_scopes)"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: skill
priority: p1
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-SKILL-111, TASK-SKILL-112, TASK-SKILL-113, TASK-SKILL-114]
depends_on: [TASK-SKILL-113]
blocks: []

source_pages:
  - "[SKILL Appendix L](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (SKILL_BUNDLE_RUBRIC.md)"
  - Task-audit skill
  - "[SKILL Appendix J](https://cyberos-wiki.cyberskill.world/modules/skill/appendices.html) (ANTHROPIC_GUIDE_DIGEST.md)"
source_decisions:
  - DEC-091 (host-portability — CCSM is source of truth)
  - DEC-182 (frontmatter schema versioned v1 frozen)

language: python + yaml + markdown
service: modules/skill/  (134 production SKILL.md sweep) + modules/cuo/cuo/  (placeholder detector) + tools/sweep-placeholders/
new_files:
  - tools/sweep-placeholders/detect.py
  - tools/sweep-placeholders/suggest.py
  # auto-generated catalog of stale placeholders + suggested fix per skill
  - tools/sweep-placeholders/report.md
  - modules/cuo/cuo/placeholder_check.py
  - modules/cuo/tests/test_placeholder_check.py
modified_files:
  # add SKB-030 placeholder-free-frontmatter rule
  - website docs (SKILL Appendix L)
  # §3.13 38f mentions placeholder rule
  - Task-audit skill
  # operator-attested per-skill substitution
  - modules/skill/<each of 134 production SKILL.md files>
allowed_tools:
  - file_read: modules/skill/**, tools/sweep-placeholders/**, docs/tasks/skill/**
  - file_write: modules/skill/**, tools/sweep-placeholders/**, modules/cuo/{cuo,tests}/**, docs/tasks/skill/**
  - bash: cd tools/sweep-placeholders && python3 detect.py --report
  - bash: cd tools/sweep-placeholders && python3 suggest.py <skill_path>
disallowed_tools:
  - auto-substitute placeholders without operator review (the right substitute depends on what the skill actually does; auto-substitution risks invalidating audit trails)
  - escape brackets via the TASK-SKILL-113 migrate.sh path (that's the wrong fix — these are placeholders that need REAL values, not escape-encoding)
  - touch body XML form `<untrusted_content source="...">…</untrusted_content>` (body XML is the runtime wrapper per TASK-SKILL-113; never touched)

effort_hours: 16
subtasks:
  - "1.0h: detect.py — Python scanner that walks modules/skill/**/SKILL.md, parses frontmatter, identifies fields where YAML values contain literal `<placeholder>` patterns (not the wrap_in_marker which is post-TASK-113 already string-form)"
  - "1.5h: suggest.py — per-skill suggestion engine that reads SKILL.md body + STANDALONE_INTERVIEW + MANIFEST_SCHEMA + acceptance fixtures, proposes a substitution for each placeholder based on what the skill actually does (e.g. metadata.stage → 'b' if body references SDP stage b, description placeholders → inferred from CONTRACT_ECHO block, etc.)"
  - "1.5h: report.md generator — produces a single markdown report listing each of the 134 skills, its stale placeholders, and the suggest.py recommended substitution. Operator reviews + approves in one pass"
  - "2.0h: placeholder_check.py + tests — Python validator that fires the SKB-030 rule at audit time; identifies stale placeholders + emits structured errors with skill_path + field_path + placeholder_token"
  - "1.0h: SKB-030 rule entry in SKILL_BUNDLE_RUBRIC.md (severity: warning on draft, error on accepted+; auto_fix: never)"
  - "0.5h: task-audit skill §3.13 rule 38f"
  - "8.0h: 134-skill mechanical sweep — apply approved substitutions from report.md to each SKILL.md. ~3.5 min per skill at 134 skills. Operator can batch by persona (cluster similar skills) to amortise context-switching"
  - "0.5h: verify.py — post-sweep invariant check; assert zero placeholder-style angle brackets in any production SKILL.md frontmatter value"
risk_if_skipped: "The 134 production SKILL.md files inherited template-syntax `<placeholder>` from earlier scaffold runs that never substituted real values. Three concrete consequences: (1) Anthropic-host transpilation (Phase B per TASK-SKILL-103) will reject the frontmatter at load time per Reference B 'forbidden frontmatter chars' — even though TASK-SKILL-113 fixed `wrap_in` specifically, the broader rule SKB-040 (no-xml-in-frontmatter) fires on every remaining placeholder. The 134 skills will fail to load on any non-CyberOS host. (2) Operator UX is degraded: the audit reports + run-time error messages will show `<SDP §2 stage letter or \"cross\">` etc. instead of meaningful values, making debugging harder. (3) TASK-SKILL-111's description-format check (SKB-022 verb-stem; SKB-023 trigger phrases) is undermined when the description itself contains a `<placeholder>` block instead of substantive prose — the trigger-phrase detector treats the placeholder as a literal phrase, leaving the skill un-triggerable. Cost of THIS task ≈ 16 hours (1.5h of tooling + 8h of mechanical sweep + 6.5h of review/test/QA). Cost of NOT shipping ≈ 134 silent portability failures + a future emergency sweep under deadline pressure when the first partner connector ships."
---

## §1 — Description (BCP-14 normative)

This task catalogues, classifies, and substitutes the stale `<placeholder>` syntax that survives in 134 production SKILL.md frontmatter fields across `modules/skill/`. These placeholders were inherited from earlier scaffold runs (pre-2026-05-19 template work) and never substituted with real values. They block Anthropic-host portability (Phase B transpilers fail on Reference B's bracket prohibition) and degrade operator UX in audit + runtime error messages.

1. The detection script `tools/sweep-placeholders/detect.py` **MUST** walk every `SKILL.md` under `modules/skill/`, parse the YAML frontmatter, and identify every field whose value contains a literal `<word>` pattern that is **NOT** the now-corrected `wrap_in_marker:` (per TASK-SKILL-113). Output: a structured JSON dump grouped by skill_path, field_path, and placeholder_token.
2. Known stale-placeholder fields (from the 2026-05-19 sweep audit) **MUST** be covered: `metadata.stage` (134 hits — `<SDP §2 stage letter or "cross">`), `description` (28 hits — `<input>`, `<artifact>` etc.), `allowed_memory_scopes.write[*]` (16 hits — `<scope-glob>`), `name` (2 hits — `<artifact>-author`/`<artifact>-audit` in `_template/*` scaffolds only), and `depends_on_contracts[*].{id, pin_path}` (4 hits — `<artifact>`).
3. The substitution engine `tools/sweep-placeholders/suggest.py` **MUST** propose a concrete substitution per placeholder by reading: (a) the skill's `SKILL.md` body's CONTRACT_ECHO block (which often names the real artefact type), (b) `STANDALONE_INTERVIEW.md` (which lists real input field names), (c) `references/MANIFEST_SCHEMA.md` (which carries the real artefact id), (d) the persona's `MODULE.md` entry (which lists the SDP stage letter). Suggestions are advisory; operator review is mandatory.
4. The substitution engine **MUST NOT** auto-apply suggestions. The right substitute depends on operator domain knowledge that the engine can approximate but not guarantee. The engine emits a single `tools/sweep-placeholders/report.md` listing every skill + every placeholder + the recommended fix; operator reviews + edits the report; the sweep step then applies the operator-approved values.
5. The auditor rule **MUST** be `SKB-030 placeholder-free-frontmatter` (added to `SKILL_BUNDLE_RUBRIC.md`) with severity `warning` for `status: draft` skills and severity `error` for `status: accepted` or higher. Auto-fix: never (operator-attestation required).
6. The Python validator `cuo.placeholder_check.scan(skill_path) -> ScanResult` **MUST** return a list of `PlaceholderHit(field_path: str, value: str, suggested_substitution: str | None)`. The CI gate runs `python -m cuo.placeholder_check --catalog modules/skill/` and exits non-zero if any production skill has hits.
7. The sweep timing **SHOULD** be done in persona-grouped batches rather than one mega-commit (134 files in one commit makes diff review impractical). Recommended batches: P0 personas first (cpo, cto — ~12 skills), then P1 personas (ceo, coo, cfo, chro, cseco, clo, caio — ~30 skills), then P2+ in order. Each batch ships as a separate commit with `chore(skill): SKB-030 sweep — <persona-name> (<N> skills)` message.
8. The body XML form `<untrusted_content source="..." page="...">…</untrusted_content>` **MUST NOT** be touched anywhere. SKB-030 fires only on frontmatter values; body markup is preserved per TASK-SKILL-113 §1 #3.
9. Each per-skill substitution **MUST** preserve the SKILL.md's audit hash chain compatibility. The substitution is a frontmatter-only edit; body bytes are unchanged. Where a fine-tune signal triggers as a side effect (e.g. `skill_version` MINOR bump per AGENTS.md §11), the operator follows the standard fine-tune cycle.
10. Per-skill substitutions **MUST** be operator-attested via a `chore(skill):` commit message containing a one-line rationale per field (e.g. `metadata.stage: 'b' (SDP §2 Requirements — body §3 PLAN phase references stage b)`).
11. The CI gate **MUST** run `cuo.placeholder_check --catalog modules/skill/ --fail-on-error` on every PR after the sweep is complete. Until the sweep finishes, the gate runs as `--fail-on-error-status-accepted-only` so scaffold/draft skills don't block PRs.
12. Pre-existing exceptions: the `_template/author/SKILL.md` and `_template/audit/SKILL.md` files are intentional scaffolds and **MUST** retain their `<artifact>` placeholders (they're literal substitution tokens for `cp` operations, not real values). The detector exempts paths under `_template/`.
13. The post-sweep verify script `tools/sweep-placeholders/verify.py` **MUST** assert: zero stale-placeholder hits in any non-`_template/` SKILL.md; the placeholder_check validator agrees; every modified file still parses as valid YAML.
14. The sweep report **MUST** be committed alongside the substitutions (lives at `tools/sweep-placeholders/report-<YYYY-MM-DD>.md`). Future operators reviewing the sweep can reconstruct the decision chain.
15. Registry version: this task is a v0.2.6 increment (post-v0.2.5 introduced by TASK-SKILL-113). The bump is documented in the repo-root `CHANGELOG.md` `[SKILL]` section.

## §2 — Why this design (rationale for humans)

**Why a separate task rather than fold into TASK-SKILL-113 (§1 #1)?** TASK-SKILL-113 specifically migrated the `wrap_in` field via a mechanical pattern match. The stale placeholders span DOZENS of distinct fields with NO common pattern. Each one needs operator domain knowledge to substitute correctly. Bundling the work into 113 would have either inflated 113 from 12h to 28h (wrong scope grouping) or shipped 113 incomplete (leaving 134 portability bugs). Separating the tasks honours the audit-fix-audit discipline — 113 closed one specific bug class; 115 closes the residual.

**Why operator-attested rather than auto-substituted (§1 #4)?** Each placeholder asks a domain question. `metadata.stage: <SDP §2 stage letter or "cross">` — is this skill stage b (requirements), c (design), e (delivery), or cross-cutting? The right answer is in the skill's body, but extracting it requires understanding what the skill actually does. A pattern-matching auto-substituter would get this wrong 30-50% of the time (the body sometimes mentions multiple stages; some skills genuinely span multiple stages and should be "cross"). Operator review at 3.5 min/skill × 134 skills = 8 hours is the right tradeoff.

**Why suggestions rather than fully manual (§1 #3)?** Pure-manual would cost 15-20 min/skill × 134 = 30-45 hours. With suggestions, operator reads the suggestion + verifies via body skim (3-5 min/skill). 4× speedup with negligible correctness loss because the suggestion engine cites its sources (CONTRACT_ECHO, MANIFEST_SCHEMA, MODULE.md) — operator verifies by skim, not by re-derivation.

**Why severity warning on draft + error on accepted (§1 #5)?** Drafting is iterative; forcing the rule on every draft commit would slow first-pass authoring. Production skills are routed by the supervisor and shipped to hosts — they MUST conform. Same severity scheme as SKB-020..023 + SKB-050..057 (TASK-SKILL-111 + 112) for consistency.

**Why persona-grouped batches (§1 #7)?** Three reasons. (1) Diff review: a 134-file commit is unreviewable; 12-30 file batches are. (2) Risk isolation: a mistake in one batch is fixable without rolling back the rest. (3) Schedule fit: persona owners can review their own persona's sweep — distributes the review load.

**Why body XML preserved (§1 #8)?** Restates TASK-SKILL-113's invariant. The body XML is the runtime wrapper; the frontmatter is the declaration. Touching body XML would break wrapping semantics for every production skill that processes external bytes (i.e. all of them).

**Why operator-attested commit messages (§1 #10)?** Audit-chain integrity. Six months from now, when an operator wonders "why does this skill have `metadata.stage: c` instead of `b`?", the commit message has the one-line rationale. Without it, the substitution is unprovenanced — operators have to re-derive the reason from skill body context, defeating the speed-up.

**Why `_template/` exemption (§1 #12)?** The template files DELIBERATELY carry `<artifact>` placeholders. They're scaffolds; the placeholders are part of the contract for `cp -r _template/author/ <new-skill>/` followed by sed-substitution. Sweeping them would break the scaffold-and-substitute workflow.

**Why a v0.2.6 increment (§1 #15)?** Per DEC-182, frontmatter changes that consumers can detect at load time are MINOR-compatible. The placeholder sweep doesn't change schema (no field renames); it just substitutes literal values. Strictly, this is a content edit, not a registry change — but bumping registry to v0.2.6 makes the change traceable in changelogs and signals to downstream consumers that the catalog has been audit-swept.

## §3 — API contract

### Detection script — `tools/sweep-placeholders/detect.py`

```python
"""Detect stale <placeholder> syntax in modules/skill/**/SKILL.md frontmatter."""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path
import yaml

PLACEHOLDER_RE = re.compile(r"<([a-zA-Z][a-zA-Z0-9_§ /\"|.()-]*)>")
EXEMPT_PATHS = ("_template/",)

def find_hits_in_value(field_path: str, value, hits: list) -> None:
    if isinstance(value, str):
        for m in PLACEHOLDER_RE.finditer(value):
            tok = m.group(1)
            # Whitelist <br> only (mermaid line-break in body; never expected in frontmatter)
            if tok.lower() == "br":
                continue
            hits.append({"field": field_path, "value": value[:120], "token": tok})
    elif isinstance(value, dict):
        for k, v in value.items():
            find_hits_in_value(f"{field_path}.{k}", v, hits)
    elif isinstance(value, list):
        for i, v in enumerate(value):
            find_hits_in_value(f"{field_path}[{i}]", v, hits)

def scan(skill_path: Path) -> dict:
    text = skill_path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return {"skill": str(skill_path), "hits": [], "error": "no_frontmatter"}
    end = text.index("\n---\n", 4)
    fm = yaml.safe_load(text[4:end])
    hits: list = []
    find_hits_in_value("root", fm, hits)
    return {"skill": str(skill_path), "hits": hits}

def main() -> int:
    catalog_root = Path("modules/skill")
    results = []
    for f in sorted(catalog_root.glob("**/SKILL.md")):
        if any(part in str(f) for part in EXEMPT_PATHS):
            continue
        r = scan(f)
        if r.get("hits"):
            results.append(r)
    print(json.dumps({"total_skills_with_hits": len(results), "skills": results}, indent=2))
    return 0 if not results else 1

if __name__ == "__main__":
    sys.exit(main())
```

### Suggestion engine — `tools/sweep-placeholders/suggest.py`

```python
"""Per-skill suggestion engine: propose substitutions based on body + sibling files."""
import re
from pathlib import Path

def suggest_for_metadata_stage(skill_path: Path) -> str | None:
    """Read skill body, find SDP stage references, return the most-frequent stage letter."""
    text = (skill_path / "SKILL.md").read_text(encoding="utf-8")
    stages = re.findall(r"stage[s]?\s+([a-h])\b", text, re.IGNORECASE)
    if not stages:
        return None
    from collections import Counter
    counts = Counter(s.lower() for s in stages)
    most_common = counts.most_common(1)[0][0]
    return most_common

def suggest_for_description(skill_path: Path, field_value: str) -> str | None:
    """Find <input>, <artifact> placeholders + propose concrete substitutions."""
    # Read CONTRACT_ECHO block for artefact type
    text = (skill_path / "SKILL.md").read_text(encoding="utf-8")
    m = re.search(r"template_version:\s+([a-z][a-z_0-9-]*)@1", text)
    artefact = m.group(1) if m else None
    if not artefact:
        return None
    # Substitute <input> → artefact, <artifact> → artefact in description
    new_value = field_value
    if "<input>" in new_value and artefact:
        new_value = new_value.replace("<input>", f"{artefact} source")
    if "<artifact>" in new_value:
        new_value = new_value.replace("<artifact>", artefact)
    return new_value

# Full suggest() function dispatches by field path; per-field heuristics.
```

### Auditor rule — addition to `SKILL_BUNDLE_RUBRIC.md`

```markdown
### SKB-030 — placeholder-free-frontmatter

**Statement:** No SKILL.md frontmatter field value may contain literal placeholder syntax like `<word>` (excluding the explicitly-allowed `<br>` if it ever appears, which it shouldn't in frontmatter). Per TASK-SKILL-115. This is distinct from SKB-040 (no-xml-in-frontmatter, which targets the security boundary); SKB-030 targets the operator-UX + portability boundary.

**Severity:** error on `status: accepted | building | shipped`; warning on `status: draft`. Exempt: any path under `_template/`.

**Auto-fix:** never (operator-attestation required — see TASK-SKILL-115 §1 #4).

**Check:** `python -m cuo.placeholder_check <skill_path>`; exit 0 if clean, exit 1 with structured error otherwise.
```

## §4 — Acceptance criteria

1. **detect.py finds known stale placeholders** — `python3 tools/sweep-placeholders/detect.py` on the catalog as-of-2026-05-19 reports exactly the 188 occurrences across 134 files (per the ANTHROPIC_GUIDE_DIGEST.md verification).
2. **detect.py exempts _template/** — running detect.py reports zero hits in `_template/author/SKILL.md` or `_template/audit/SKILL.md` (those are scaffolds with intentional placeholders).
3. **detect.py whitelists `<br>`** — a `<br>` in any frontmatter value (theoretical; shouldn't happen but defended) does not register as a hit.
4. **suggest.py proposes a stage letter** — for `task-author` (body references SDP §2(b) Requirements), suggest_for_metadata_stage returns `'b'`.
5. **suggest.py reads CONTRACT_ECHO for artefact type** — for `task-author`, the description's `<input>` → `task source`.
6. **placeholder_check.scan returns structured PlaceholderHit list** — given a skill with `metadata.stage: <SDP §2 stage letter or "cross">`, returns one hit with `field_path='root.metadata.stage'` + `value='<SDP §2 stage letter or "cross">'` + `suggested_substitution='b'` (or None if suggest fails).
7. **placeholder_check exit code 1 on hits** — running on a skill with stale placeholders returns exit code 1; running on a clean skill returns 0.
8. **SKB-030 rule fires on production skill with placeholders** — given a `status: accepted` skill with stale placeholders, the auditor reports one SKB-030 issue (severity error, status needs_human).
9. **SKB-030 rule fires as warning on draft skill** — same input but `status: draft` → severity warning.
10. **Body XML preserved post-sweep** — after running the full sweep, every body's `<untrusted_content source="..." page="...">…</untrusted_content>` markup is unchanged byte-for-byte.
11. **All 134 production skills are placeholder-free post-sweep** — `python3 tools/sweep-placeholders/detect.py` exits 0; verify.py confirms.
12. **Persona-batch commits are reviewable** — each batch commit's diff is ≤30 files; commit message follows `chore(skill): SKB-030 sweep — <persona-name> (<N> skills)` format.
13. **Operator-attestation in commit message** — every batch commit includes one-line rationale per field-type substitution (e.g. "metadata.stage values picked from each skill's body SDP-stage references; cross-cutting skills marked 'cross'").
14. **verify.py asserts post-sweep invariants** — running verify.py exits 0; checks (a) detect.py reports zero hits, (b) every modified file parses as valid YAML, (c) every modified file's `wrap_in_marker:` field is still `"untrusted_content"` (TASK-SKILL-113 invariant preserved), (d) body XML form unchanged via SHA256 comparison.
15. **CI gate integration** — `python -m cuo.placeholder_check --catalog modules/skill/ --fail-on-error` runs as part of the existing CUO test suite; PRs touching SKILL.md frontmatter are gated.
16. **task-audit skill §3.13 rule 38f added** — references TASK-SKILL-115 + SKB-030.
17. **Registry version bumped** — v0.2.5 → v0.2.6 in CHANGELOG.md `[SKILL]` section.
18. **Sweep report committed** — `tools/sweep-placeholders/report-2026-05-19.md` (or later date) lists every skill + every substitution decision.
19. **Idempotency check** — running detect.py + sweep + detect.py again yields the same zero-hit state; no flapping.
20. **Cross-task reciprocity** — TASK-SKILL-113's `blocks:` list updated to include TASK-SKILL-115; reciprocity sweep passes.

## §5 — Verification

```python
# modules/cuo/tests/test_placeholder_check.py
import pytest
from pathlib import Path
from cuo.placeholder_check import scan, PlaceholderHit

def test_detects_metadata_stage_placeholder(tmp_path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo-author
metadata:
  stage: <SDP §2 stage letter or "cross">
wrap_in_marker: "untrusted_content"
---
body
""", encoding="utf-8")
    result = scan(skill)
    assert len(result.hits) >= 1
    assert any(h.field_path.endswith(".stage") for h in result.hits)

def test_ignores_template_path(tmp_path):
    tpath = tmp_path / "_template" / "author"
    tpath.mkdir(parents=True)
    skill = tpath / "SKILL.md"
    skill.write_text("""---
name: <artefact>-author
metadata:
  stage: <SDP §2 stage letter>
---
body
""", encoding="utf-8")
    result = scan(skill)
    # Template path → return empty hits (exempt)
    assert result.exempt is True

def test_whitelists_br_tag(tmp_path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo
description: "Line 1<br>Line 2"
---
""", encoding="utf-8")
    result = scan(skill)
    assert len(result.hits) == 0  # <br> exempt

def test_wrap_in_marker_not_flagged(tmp_path):
    skill = tmp_path / "SKILL.md"
    skill.write_text("""---
name: foo
metadata:
  stage: b
wrap_in_marker: "untrusted_content"
---
body
""", encoding="utf-8")
    result = scan(skill)
    assert len(result.hits) == 0
```

## §6 — Implementation skeleton

§3 covers the surface. Wiring:

1. `tools/sweep-placeholders/detect.py` is the catalog scanner.
2. `tools/sweep-placeholders/suggest.py` is the per-skill suggestion engine.
3. `tools/sweep-placeholders/report.md` is the auto-generated report; operator edits to approve.
4. `modules/cuo/cuo/placeholder_check.py` is the runtime validator (re-exported from `cuo` package).
5. `modules/cuo/tests/test_placeholder_check.py` integrates with the existing CUO test suite.
6. SKILL_BUNDLE_RUBRIC.md gains SKB-030.
7. Task-audit skill gains §3.13 rule 38f.
8. Per-persona batches commit independently (P0 cpo + cto first, then P1, then P2+).

## §7 — Dependencies

**Depends on:**
- **TASK-SKILL-113** (XML-free frontmatter) — provides the `wrap_in_marker:` rename so the detector can correctly distinguish the now-fixed marker from stale placeholders.

**Blocks:** none (independent of TASK-SKILL-114; orthogonal to TASK-SKILL-111 + 112).

**Related:** all of TASK-SKILL-111 / 112 / 113 / 114 — together they form the v0.2.5/v0.2.6 portability + foundation-discipline bundle.

## §8 — Example payloads

### Example 1 — detect.py output (truncated)

```json
{
  "total_skills_with_hits": 134,
  "skills": [
    {
      "skill": "modules/skill/transformation-roadmap-audit/SKILL.md",
      "hits": [
        {"field": "root.metadata.stage", "value": "<SDP §2 stage letter or \"cross\">", "token": "SDP §2 stage letter or \"cross\""}
      ]
    }
  ]
}
```

### Example 2 — suggest.py output

```
$ python3 tools/sweep-placeholders/suggest.py modules/skill/transformation-roadmap-audit
Field: root.metadata.stage
Current: <SDP §2 stage letter or "cross">
Suggested: "cross"
Rationale: Body references stages b, d, e — multi-stage skill; choose "cross".
```

### Example 3 — Audit issue (SKB-030 firing)

```
ISSUE
id:              ISS-014
rule_id:         SKB-030
severity:        error
category:        placeholder_in_frontmatter
location:        frontmatter root.metadata.stage
evidence:        "metadata.stage: <SDP §2 stage letter or \"cross\">"
description:     "Frontmatter field metadata.stage contains stale template placeholder syntax. Per TASK-SKILL-115 §1 #1, every production skill must carry concrete substituted values."
suggestion:      "Run 'python3 tools/sweep-placeholders/suggest.py modules/skill/transformation-roadmap-audit' for a context-aware recommendation. The body references SDP stages b/d/e — suggest 'cross'."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-05-19T17:00:00Z"
updated_at:      "2026-05-19T17:00:00Z"
```

## §9 — Open questions

**All resolved during authoring.**

Deferred:
- **Per-persona owner sign-off** at batch-commit time — out of scope for this task; handled by normal PR review process.
- **Automated batch-commit grouping by persona** — out of scope; the operator manually groups via `git add modules/skill/<persona>/`.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| suggest.py proposes wrong value | Operator review catches at report.md stage | No bad substitution shipped | Edit report.md; re-run sweep |
| Operator approves wrong substitution despite review | Audit log captures the wrong value; next fine-tune cycle on the skill surfaces the issue | Bad value lives until fine-tune | Standard fine-tune cycle corrects |
| Sweep accidentally touches body XML | verify.py SHA256-compares pre/post body bytes; mismatch → exit 1 | Sweep batch rejected | Re-run with operator review of diff |
| 134-skill sweep done in one mega-commit | Diff review fails (too large) | PR blocked | Split into persona batches |
| Migration runs while a fine-tune is mid-flight on the same skill | Conflict at commit; standard git merge resolves | Operator resolves conflict per file | Reapply suggested substitution + re-run verify |
| SKB-030 rule fires on draft skill mid-authoring | Severity warning only — author iterates without friction | No production impact | Substitute placeholder when promoting to accepted |
| _template/ exemption mis-fires (skips non-template skill) | detect.py output is incomplete | False clean signal | EXEMPT_PATHS path-prefix check is anchored to `_template/`; verify with `grep -l '_template/' tools/sweep-placeholders/detect.py` |
| Registry version bump conflicts with concurrent TASK-SKILL-117 marker namespace work | Operator coordinates timing | Version bump skipped | Bump on TASK-SKILL-117 commit instead |
| Placeholder appears in body XML (false positive) | Detector only scans frontmatter; body bytes untouched | No false positive | Detector splits at `\n---\n` and only parses frontmatter |
| Operator misreads a multi-line YAML folded scalar as a placeholder | suggest.py shows full flattened context; operator confirms before approval | Edge case caught at review | Adjust report.md template to render flattened equivalent alongside raw |
| CI gate fires before sweep finishes | `--fail-on-error-status-accepted-only` flag protects scaffold/draft skills during transition | PR not blocked on legitimate draft work | Operator transitions flag to `--fail-on-error` post-sweep completion |
| `wrap_in_marker:` accidentally regressed during sweep | verify.py invariant (c) catches: marker still must equal `"untrusted_content"` | Sweep batch rejected | Re-run with corrected substitution |
| Multi-stage skill ambiguity (which letter to pick?) | suggest.py recommends "cross"; operator can override | No silent miscategorisation | Operator review at report.md stage |
| Body references stage letter that doesn't exist in SDP (typo) | suggest.py returns None; operator must hand-edit | No silent bad value | Hand-edit + commit |

## §11 — Implementation notes

- **Why persona-grouped batches over single mega-commit?** Three reasons (already in §2): diff reviewability, risk isolation, schedule fit. Persona batches also align with the existing CHANGELOG taxonomy.
- **Why preserve `_template/` placeholders deliberately?** They're scaffolds. Removing them would break `cp -r _template/author/ <new-skill>/` + sed-substitute workflow. The template files DOCUMENT what placeholders go where — they're contract artefacts, not stale leakage.
- **Why the broad placeholder regex (allowing §, /, ", etc. inside)?** The actual stale values in the catalog include `<SDP §2 stage letter or "cross">` — quite verbose. A naive `<\w+>` regex would miss the multi-word placeholder. The chosen regex matches anything that LOOKS like an angle-bracket placeholder text fragment.
- **Why suggest.py reads multiple sibling files (CONTRACT_ECHO, MANIFEST_SCHEMA, MODULE.md)?** Different fields' correct substitutions come from different sources. metadata.stage comes from body SDP-stage references; description placeholders come from CONTRACT_ECHO template_version; depends_on_contracts come from MANIFEST_SCHEMA. One source-of-truth per field-type.
- **Why operator-attested commit messages instead of an automated audit row?** Both happen. The commit message is human-readable rationale; the audit row (per AGENTS.md §7) is the machine-readable record. Together they form the chain of custody for each substitution.
- **Why 8 hours of mechanical sweep across 134 skills?** Math: 3.5 minutes per skill = ~12 skills/hour. 134 / 12 ≈ 11 hours pre-suggestions; with suggest.py shortening review to ~2 min/skill, 8 hours is realistic.
- **Why dependency on TASK-SKILL-113 instead of independent?** TASK-SKILL-113 introduced `wrap_in_marker:` — without that rename in place, the detector might confuse the OLD `wrap_in: <untrusted_content/>` form (which 113 already migrated) with stale placeholders. TASK-SKILL-113 first; then TASK-SKILL-115 picks up the residual.

---

*End of TASK-SKILL-115.*
